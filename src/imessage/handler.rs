use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::{debug, error, info, warn};

use super::{ChatInfo, Contact, IMessageAPI, Message, ReadReceipt, SendMessageStatus, TypingNotification};

const PORTAL_BUFFER_TIMEOUT: Duration = Duration::from_secs(10);
const CHANNEL_SIZE: usize = 1000;

#[derive(Debug, Clone)]
pub enum IMessageEvent {
    Message(Box<Message>),
    ReadReceipt(ReadReceipt),
    TypingNotification(TypingNotification),
    Chat(Box<ChatInfo>),
    Contact(Box<Contact>),
    MessageStatus(SendMessageStatus),
    BackfillTask(BackfillTask),
}

#[derive(Debug, Clone)]
pub struct BackfillTask {
    pub chat_guid: String,
    pub backfill_id: String,
    pub messages: Vec<Message>,
}

pub struct IMessageHandler {
    event_rx: Receiver<IMessageEvent>,
    event_tx: Sender<IMessageEvent>,
    bridge: Arc<dyn IMessageBridge + Send + Sync>,
    stop_tx: Option<Sender<()>>,
}

#[async_trait::async_trait]
pub trait IMessageBridge: Send + Sync {
    async fn get_portal_by_guid(&self, chat_guid: &str) -> Option<Arc<dyn Portal + Send + Sync>>;
    async fn get_puppet_by_guid(&self, user_guid: &str) -> Option<Arc<dyn Puppet + Send + Sync>>;
    async fn send_read_receipt(&self, portal: &str, rr: ReadReceipt);
    async fn send_typing_notification(&self, portal: &str, notif: TypingNotification);
    async fn send_message(&self, portal: &str, msg: Message);
    async fn create_matrix_room(&self, chat_info: &ChatInfo) -> Result<(), Box<dyn std::error::Error>>;
    async fn is_backfill_enabled(&self) -> bool;
    async fn send_backfill_result(&self, chat_guid: &str, backfill_id: &str, success: bool);
}

#[async_trait::async_trait]
pub trait Portal: Send + Sync {
    fn guid(&self) -> &str;
    fn mxid(&self) -> Option<&str>;
    async fn send_message(&self, msg: Message) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_read_receipt(&self, rr: ReadReceipt) -> Result<(), Box<dyn std::error::Error>>;
    async fn send_message_status(&self, status: SendMessageStatus) -> Result<(), Box<dyn std::error::Error>>;
    async fn sync_with_info(&self, chat_info: &ChatInfo);
    async fn update_bridge_info(&self);
}

#[async_trait::async_trait]
pub trait Puppet: Send + Sync {
    fn guid(&self) -> &str;
    fn mxid(&self) -> Option<&str>;
    async fn sync_with_contact(&self, contact: &Contact);
}

impl IMessageHandler {
    pub fn new(bridge: Arc<dyn IMessageBridge + Send + Sync>) -> Self {
        let (event_tx, event_rx) = mpsc::channel(CHANNEL_SIZE);
        
        Self {
            event_rx,
            event_tx,
            bridge,
            stop_tx: None,
        }
    }

    pub fn event_sender(&self) -> Sender<IMessageEvent> {
        self.event_tx.clone()
    }

    pub async fn start(&mut self) {
        let (stop_tx, mut stop_rx) = mpsc::channel(1);
        self.stop_tx = Some(stop_tx);

        info!("Starting iMessage handler");

        loop {
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    let start = Instant::now();
                    let event_type = self.handle_event(event).await;
                    let elapsed = start.elapsed();
                    
                    debug!(
                        "Handled {} in {:?} (queued: {})",
                        event_type,
                        elapsed,
                        self.event_rx.capacity() - self.event_rx.max_capacity()
                    );
                }
                
                _ = stop_rx.recv() => {
                    info!("Stopping iMessage handler");
                    break;
                }
            }
        }
    }

    async fn handle_event(&self, event: IMessageEvent) -> &'static str {
        match event {
            IMessageEvent::Message(msg) => {
                self.handle_message(*msg).await;
                "message"
            }
            IMessageEvent::ReadReceipt(rr) => {
                self.handle_read_receipt(rr).await;
                "read receipt"
            }
            IMessageEvent::TypingNotification(notif) => {
                self.handle_typing_notification(notif).await;
                "typing notification"
            }
            IMessageEvent::Chat(chat) => {
                self.handle_chat(*chat).await;
                "chat"
            }
            IMessageEvent::Contact(contact) => {
                self.handle_contact(*contact).await;
                "contact"
            }
            IMessageEvent::MessageStatus(status) => {
                self.handle_message_status(status).await;
                "message status"
            }
            IMessageEvent::BackfillTask(task) => {
                self.handle_backfill_task(task).await;
                "backfill task"
            }
        }
    }

    async fn handle_message(&self, msg: Message) {
        debug!("Received incoming message {} in {}", msg.guid, msg.chat_guid);
        
        if let Some(portal) = self.bridge.get_portal_by_guid(&msg.chat_guid).await {
            if portal.mxid().is_none() {
                info!("Creating Matrix room to handle message");
                let chat_info = ChatInfo {
                    guid: msg.chat_guid.clone(),
                    ..Default::default()
                };
                if let Err(e) = self.bridge.create_matrix_room(&chat_info).await {
                    warn!("Failed to create Matrix room to handle message: {}", e);
                    return;
                }
            }

            match portal.send_message(msg).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to send message to portal: {}", e);
                }
            }
        } else {
            warn!("No portal found for chat GUID: {}", msg.chat_guid);
        }
    }

    async fn handle_read_receipt(&self, rr: ReadReceipt) {
        debug!("Received read receipt in {}", rr.chat_guid);
        
        if let Some(portal) = self.bridge.get_portal_by_guid(&rr.chat_guid).await {
            if portal.mxid().is_some() {
                if let Err(e) = portal.send_read_receipt(rr).await {
                    error!("Failed to send read receipt to portal: {}", e);
                }
            } else {
                debug!("Ignoring read receipt in unknown portal {}", rr.chat_guid);
            }
        }
    }

    async fn handle_typing_notification(&self, notif: TypingNotification) {
        debug!("Received typing notification in {}", notif.chat_guid);
        
        self.bridge.send_typing_notification(&notif.chat_guid, notif).await;
    }

    async fn handle_chat(&self, chat: ChatInfo) {
        debug!("Received chat update for {}", chat.guid);
        
        if let Some(portal) = self.bridge.get_portal_by_guid(&chat.guid).await {
            if portal.mxid().is_some() {
                info!("Syncing Matrix room to handle chat command");
                portal.sync_with_info(&chat).await;
                portal.update_bridge_info().await;
            } else if !chat.no_create_room {
                info!("Creating Matrix room to handle chat command");
                if let Err(e) = self.bridge.create_matrix_room(&chat).await {
                    warn!("Failed to create Matrix room to handle chat command: {}", e);
                }
            }
        }
    }

    async fn handle_contact(&self, contact: Contact) {
        debug!("Received contact update for {}", contact.user_guid);
        
        if let Some(puppet) = self.bridge.get_puppet_by_guid(&contact.user_guid).await {
            if puppet.mxid().is_some() {
                info!("Syncing Puppet to handle contact command");
                puppet.sync_with_contact(&contact).await;
            }
        }
    }

    async fn handle_message_status(&self, status: SendMessageStatus) {
        debug!("Received message status for {}", status.guid);
        
        if let Some(portal) = self.bridge.get_portal_by_guid(&status.chat_guid).await {
            if portal.mxid().is_some() {
                if let Err(e) = portal.send_message_status(status).await {
                    error!("Failed to send message status to portal: {}", e);
                }
            } else {
                debug!(
                    "Ignoring message status for message from unknown portal {}/{}",
                    status.guid, status.chat_guid
                );
            }
        }
    }

    async fn handle_backfill_task(&self, task: BackfillTask) {
        debug!("Received backfill task for {}", task.chat_guid);
        
        if !self.bridge.is_backfill_enabled().await {
            warn!("Connector sent backfill task, but backfill is disabled in bridge config");
            self.bridge
                .send_backfill_result(&task.chat_guid, &task.backfill_id, false)
                .await;
            return;
        }

        if let Some(portal) = self.bridge.get_portal_by_guid(&task.chat_guid).await {
            if portal.mxid().is_none() {
                error!("Tried to backfill chat {} with no portal", portal.guid());
                self.bridge
                    .send_backfill_result(portal.guid(), &task.backfill_id, false)
                    .await;
                return;
            }

            info!("Running backfill {} in background", task.backfill_id);
            let messages = task.messages;
            let chat_guid = task.chat_guid.clone();
            let backfill_id = task.backfill_id.clone();
            let bridge = self.bridge.clone();

            tokio::spawn(async move {
                for msg in messages {
                    if let Err(e) = portal.send_message(msg).await {
                        error!("Failed to send backfill message: {}", e);
                    }
                }
                bridge.send_backfill_result(&chat_guid, &backfill_id, true).await;
            });
        }
    }

    pub async fn stop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBridge;

    #[async_trait::async_trait]
    impl IMessageBridge for MockBridge {
        async fn get_portal_by_guid(&self, _chat_guid: &str) -> Option<Arc<dyn Portal + Send + Sync>> {
            None
        }
        async fn get_puppet_by_guid(&self, _user_guid: &str) -> Option<Arc<dyn Puppet + Send + Sync>> {
            None
        }
        async fn send_read_receipt(&self, _portal: &str, _rr: ReadReceipt) {}
        async fn send_typing_notification(&self, _portal: &str, _notif: TypingNotification) {}
        async fn send_message(&self, _portal: &str, _msg: Message) {}
        async fn create_matrix_room(&self, _chat_info: &ChatInfo) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }
        async fn is_backfill_enabled(&self) -> bool {
            false
        }
        async fn send_backfill_result(&self, _chat_guid: &str, _backfill_id: &str, _success: bool) {}
    }

    #[tokio::test]
    async fn test_handler_creation() {
        let bridge = Arc::new(MockBridge);
        let handler = IMessageHandler::new(bridge);
        assert!(handler.stop_tx.is_none());
    }
}
