use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, error, info, warn};

use crate::db::models::Portal as DBPortal;
use crate::imessage::{
    ChatInfo, Contact, IMessageEvent, Message, ReadReceipt, SendMessageStatus, Tapback,
    TypingNotification,
};
use crate::matrix::MatrixClient;

const MESSAGE_CHANNEL_SIZE: usize = 100;
const BACKFILL_QUEUE_SIZE: usize = 50;

#[derive(Debug, Clone)]
pub struct PortalIdentifier {
    pub guid: String,
    pub service: String,
    pub is_group: bool,
}

impl PortalIdentifier {
    pub fn parse(guid: &str) -> Self {
        let parts: Vec<&str> = guid.splitn(3, ';').collect();
        if parts.len() >= 3 {
            Self {
                service: parts[0].to_string(),
                is_group: parts[1] == "+",
                guid: guid.to_string(),
            }
        } else {
            Self {
                guid: guid.to_string(),
                service: "iMessage".to_string(),
                is_group: false,
            }
        }
    }

    pub fn is_private_chat(&self) -> bool {
        !self.is_group
    }
}

pub struct Portal {
    db_portal: DBPortal,
    identifier: PortalIdentifier,
    matrix: Arc<MatrixClient>,
    
    messages_rx: Mutex<Option<Receiver<IMessageEvent>>>,
    messages_tx: Sender<IMessageEvent>,
    
    backfill_queue: Mutex<Vec<Message>>,
    backfill_active: Mutex<bool>,
    
    room_create_lock: Mutex<()>,
    message_dedup: RwLock<HashMap<String, (String, chrono::DateTime<chrono::Utc>)>>,
    
    user_is_typing: Mutex<bool>,
}

impl Portal {
    pub fn new(db_portal: DBPortal, matrix: Arc<MatrixClient>) -> Self {
        let identifier = PortalIdentifier::parse(&db_portal.guid);
        let (messages_tx, messages_rx) = mpsc::channel(MESSAGE_CHANNEL_SIZE);
        
        Self {
            db_portal,
            identifier,
            matrix,
            messages_rx: Mutex::new(Some(messages_rx)),
            messages_tx,
            backfill_queue: Mutex::new(Vec::new()),
            backfill_active: Mutex::new(false),
            room_create_lock: Mutex::new(()),
            message_dedup: RwLock::new(HashMap::new()),
            user_is_typing: Mutex::new(false),
        }
    }

    pub fn guid(&self) -> &str {
        &self.db_portal.guid
    }

    pub fn mxid(&self) -> Option<&str> {
        self.db_portal.mxid.as_deref()
    }

    pub fn identifier(&self) -> &PortalIdentifier {
        &self.identifier
    }

    pub fn is_private_chat(&self) -> bool {
        self.identifier.is_private_chat()
    }

    pub fn event_sender(&self) -> Sender<IMessageEvent> {
        self.messages_tx.clone()
    }

    pub async fn start_handling(&self) {
        let mut rx = self.messages_rx.lock().take();
        
        if let Some(rx) = rx {
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        IMessageEvent::Message(msg) => {
                            if let Err(e) = self.handle_imessage(*msg).await {
                                error!("Failed to handle iMessage: {}", e);
                            }
                        }
                        IMessageEvent::ReadReceipt(rr) => {
                            if let Err(e) = self.handle_read_receipt(rr).await {
                                error!("Failed to handle read receipt: {}", e);
                            }
                        }
                        IMessageEvent::TypingNotification(notif) => {
                            if let Err(e) = self.handle_typing_notification(notif).await {
                                error!("Failed to handle typing notification: {}", e);
                            }
                        }
                        IMessageEvent::MessageStatus(status) => {
                            if let Err(e) = self.handle_message_status(status).await {
                                error!("Failed to handle message status: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
            });
        }
    }

    async fn handle_imessage(&self, msg: Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Handling iMessage {} in portal {}", msg.guid, self.guid());
        
        if self.mxid().is_none() {
            info!("Creating Matrix room for portal {}", self.guid());
            self.create_matrix_room(None).await?;
        }

        let dedup_key = format!("{}:{}", msg.chat_guid, msg.guid);
        if self.message_dedup.read().contains_key(&dedup_key) {
            debug!("Skipping duplicate message {}", msg.guid);
            return Ok(());
        }

        let event_id = self.send_message_to_matrix(&msg).await?;
        
        self.message_dedup.write().insert(
            dedup_key,
            (event_id.clone(), chrono::Utc::now()),
        );
        
        self.cleanup_old_dedup_entries();

        Ok(())
    }

    async fn handle_read_receipt(&self, rr: ReadReceipt) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Handling read receipt in portal {}", self.guid());
        
        if let Some(mxid) = self.mxid() {
            self.matrix.send_read_receipt(mxid, rr).await?;
        }
        
        Ok(())
    }

    async fn handle_typing_notification(&self, notif: TypingNotification) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Handling typing notification in portal {}", self.guid());
        
        let mut user_is_typing = self.user_is_typing.lock();
        
        if *user_is_typing != notif.typing {
            if let Some(mxid) = self.mxid() {
                self.matrix.send_typing(mxid, notif.typing, Duration::from_secs(60)).await?;
                *user_is_typing = notif.typing;
            }
        }
        
        Ok(())
    }

    async fn handle_message_status(&self, status: SendMessageStatus) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Handling message status for {} in portal {}", status.guid, self.guid());
        
        if let Some(mxid) = self.mxid() {
            self.matrix.send_message_status(mxid, status).await?;
        }
        
        Ok(())
    }

    pub async fn create_matrix_room(&self, chat_info: Option<&ChatInfo>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let _lock = self.room_create_lock.lock();
        
        if let Some(mxid) = self.mxid() {
            return Ok(mxid.to_string());
        }

        info!("Creating Matrix room for portal {}", self.guid());
        
        let mxid = if let Some(info) = chat_info {
            self.matrix.create_room(info).await?
        } else {
            let default_info = ChatInfo {
                guid: self.guid().to_string(),
                ..Default::default()
            };
            self.matrix.create_room(&default_info).await?
        };

        self.db_portal.mxid = Some(mxid.clone());
        self.db_portal.save()?;

        info!("Created Matrix room {} for portal {}", mxid, self.guid());
        
        Ok(mxid)
    }

    pub async fn sync_with_info(&self, chat_info: &ChatInfo) {
        debug!("Syncing portal {} with chat info", self.guid());
        
        if let Some(mxid) = self.mxid() {
            if let Err(e) = self.matrix.sync_room_info(mxid, chat_info).await {
                error!("Failed to sync room info: {}", e);
            }
        }
    }

    pub async fn update_bridge_info(&self) {
        debug!("Updating bridge info for portal {}", self.guid());
        
        if let Some(mxid) = self.mxid() {
            if let Err(e) = self.matrix.update_bridge_info(mxid).await {
                error!("Failed to update bridge info: {}", e);
            }
        }
    }

    async fn send_message_to_matrix(&self, msg: &Message) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mxid) = self.mxid() {
            self.matrix.send_message(mxid, msg).await
        } else {
            Err("Portal has no Matrix room".into())
        }
    }

    fn cleanup_old_dedup_entries(&self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        let mut dedup = self.message_dedup.write();
        dedup.retain(|_, (_, timestamp)| *timestamp > cutoff);
    }

    pub fn add_backfill_message(&self, msg: Message) {
        let mut queue = self.backfill_queue.lock();
        queue.push(msg);
    }

    pub async fn process_backfill(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active = self.backfill_active.lock();
        
        if *active {
            return Ok(());
        }
        
        *active = true;
        drop(active);

        let messages: Vec<Message> = {
            let mut queue = self.backfill_queue.lock();
            queue.drain(..).collect()
        };

        info!("Processing {} backfill messages for portal {}", messages.len(), self.guid());

        for msg in messages {
            if let Err(e) = self.handle_imessage(msg).await {
                error!("Failed to backfill message: {}", e);
            }
        }

        *self.backfill_active.lock() = false;

        Ok(())
    }

    pub async fn handle_matrix_message(&self, msg: crate::matrix::MatrixMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Handling Matrix message in portal {}", self.guid());
        
        // TODO: Send message to iMessage
        // self.imessage.send_message(self.guid(), msg).await?;
        
        Ok(())
    }

    pub async fn handle_matrix_reaction(&self, reaction: crate::matrix::MatrixReaction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Handling Matrix reaction in portal {}", self.guid());
        
        // TODO: Send tapback to iMessage
        // self.imessage.send_tapback(self.guid(), reaction).await?;
        
        Ok(())
    }
}

pub struct PortalManager {
    portals_by_guid: RwLock<HashMap<String, Arc<Portal>>>,
    portals_by_mxid: RwLock<HashMap<String, Arc<Portal>>>,
    matrix: Arc<MatrixClient>,
}

impl PortalManager {
    pub fn new(matrix: Arc<MatrixClient>) -> Self {
        Self {
            portals_by_guid: RwLock::new(HashMap::new()),
            portals_by_mxid: RwLock::new(HashMap::new()),
            matrix,
        }
    }

    pub async fn get_portal_by_guid(&self, guid: &str) -> Option<Arc<Portal>> {
        let portals = self.portals_by_guid.read();
        portals.get(guid).cloned()
    }

    pub async fn get_portal_by_mxid(&self, mxid: &str) -> Option<Arc<Portal>> {
        let portals = self.portals_by_mxid.read();
        portals.get(mxid).cloned()
    }

    pub async fn create_or_get_portal(&self, guid: &str) -> Result<Arc<Portal>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(portal) = self.get_portal_by_guid(guid).await {
            return Ok(portal);
        }

        let db_portal = DBPortal::get_or_create(guid)?;
        let portal = Arc::new(Portal::new(db_portal, self.matrix.clone()));
        
        {
            let mut portals = self.portals_by_guid.write();
            portals.insert(guid.to_string(), portal.clone());
        }
        
        if let Some(mxid) = portal.mxid() {
            let mut portals = self.portals_by_mxid.write();
            portals.insert(mxid.to_string(), portal.clone());
        }
        
        portal.start_handling().await;

        Ok(portal)
    }

    pub async fn load_all_portals(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_portals = DBPortal::get_all_with_mxid()?;
        
        for db_portal in db_portals {
            let portal = Arc::new(Portal::new(db_portal, self.matrix.clone()));
            
            {
                let mut portals = self.portals_by_guid.write();
                portals.insert(portal.guid().to_string(), portal.clone());
            }
            
            if let Some(mxid) = portal.mxid() {
                let mut portals = self.portals_by_mxid.write();
                portals.insert(mxid.to_string(), portal.clone());
            }
            
            portal.start_handling().await;
        }

        Ok(())
    }

    pub async fn delete_portal(&self, guid: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(portal) = self.get_portal_by_guid(guid).await {
            if let Some(mxid) = portal.mxid() {
                let mut portals = self.portals_by_mxid.write();
                portals.remove(mxid);
            }
            
            {
                let mut portals = self.portals_by_guid.write();
                portals.remove(guid);
            }
            
            portal.db_portal.delete()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portal_identifier_parse() {
        let id = PortalIdentifier::parse("iMessage;-;+1234567890");
        assert_eq!(id.service, "iMessage");
        assert_eq!(id.is_group, false);
        assert_eq!(id.guid, "iMessage;-;+1234567890");
        
        let id2 = PortalIdentifier::parse("SMS;+;group123");
        assert_eq!(id2.service, "SMS");
        assert_eq!(id2.is_group, true);
    }
}
