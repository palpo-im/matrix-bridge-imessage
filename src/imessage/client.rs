use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::info;

use crate::bridge::BridgeCore;
use crate::config::PlatformConfig;

use super::bluebubbles::BlueBubblesClient;
use super::interface::{BackfillTask, IMessageAPI};
use super::mac_nosip::MacNosipClient;
use super::structs::{
    Attachment, ChatIdentifier, ChatInfo, ConnectorCapabilities, Contact, CreateGroupResponse,
    Message, MessageMetadata, ReadReceipt, RichLink, SendMessageStatus, SendResponse, TapbackType,
    TypingNotification,
};

pub struct IMessageClient {
    api: Arc<RwLock<Box<dyn IMessageAPI>>>,
    config: PlatformConfig,
    bridge: Option<Arc<BridgeCore>>,
}

impl IMessageClient {
    pub async fn new(config: &crate::config::Config) -> Result<Self> {
        let platform_config = config.platform.clone();
        let api = Self::create_api(&platform_config).await?;
        
        Ok(Self {
            api: Arc::new(RwLock::new(api)),
            config: platform_config,
            bridge: None,
        })
    }
    
    async fn create_api(config: &PlatformConfig) -> Result<Box<dyn IMessageAPI>> {
        info!("Creating iMessage API client for platform: {}", config.platform);
        
        match config.platform.as_str() {
            "bluebubbles" => {
                let client = BlueBubblesClient::new(config)?;
                Ok(Box::new(client))
            }
            "mac-nosip" => {
                let client = MacNosipClient::new(config)?;
                Ok(Box::new(client))
            }
            "mac" => {
                Err(anyhow!("Native mac connector not yet implemented"))
            }
            _ => {
                Err(anyhow!("Unknown platform: {}", config.platform))
            }
        }
    }
    
    pub async fn set_bridge(&self, bridge: Arc<BridgeCore>) {
        let mut client = self.api.write().await;
        // Store bridge reference if needed
    }
    
    pub async fn start(&self) -> Result<()> {
        let mut api = self.api.write().await;
        api.start(None).await
    }
    
    pub async fn stop(&self) -> Result<()> {
        let mut api = self.api.write().await;
        api.stop().await
    }
}

// Implement IMessageAPI trait for IMessageClient by delegating to inner api
#[async_trait]
impl IMessageAPI for IMessageClient {
    async fn start(&mut self, ready_callback: Option<Box<dyn Fn() + Send>>) -> Result<()> {
        let mut api = self.api.write().await;
        api.start(ready_callback).await
    }
    
    async fn stop(&mut self) -> Result<()> {
        let mut api = self.api.write().await;
        api.stop().await
    }
    
    async fn get_messages_since_date(
        &self,
        chat_id: &str,
        min_date: DateTime<Utc>,
        backfill_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let api = self.api.read().await;
        api.get_messages_since_date(chat_id, min_date, backfill_id).await
    }
    
    async fn get_messages_between(
        &self,
        chat_id: &str,
        min_date: DateTime<Utc>,
        max_date: DateTime<Utc>,
    ) -> Result<Vec<Message>> {
        let api = self.api.read().await;
        api.get_messages_between(chat_id, min_date, max_date).await
    }
    
    async fn get_messages_before_with_limit(
        &self,
        chat_id: &str,
        before: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<Message>> {
        let api = self.api.read().await;
        api.get_messages_before_with_limit(chat_id, before, limit).await
    }
    
    async fn get_messages_with_limit(
        &self,
        chat_id: &str,
        limit: i32,
        backfill_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let api = self.api.read().await;
        api.get_messages_with_limit(chat_id, limit, backfill_id).await
    }
    
    async fn get_chats_with_messages_after(
        &self,
        min_date: DateTime<Utc>,
    ) -> Result<Vec<ChatIdentifier>> {
        let api = self.api.read().await;
        api.get_chats_with_messages_after(min_date).await
    }
    
    async fn get_message(&self, guid: &str) -> Result<Option<Message>> {
        let api = self.api.read().await;
        api.get_message(guid).await
    }
    
    fn message_chan(&self) -> tokio::sync::broadcast::Receiver<Message> {
        // This is a bit tricky with async, we need to handle this differently
        // For now, return a dummy channel
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    fn read_receipt_chan(&self) -> tokio::sync::broadcast::Receiver<ReadReceipt> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    fn typing_notification_chan(&self) -> tokio::sync::broadcast::Receiver<TypingNotification> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    fn chat_chan(&self) -> tokio::sync::broadcast::Receiver<ChatInfo> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    fn contact_chan(&self) -> tokio::sync::broadcast::Receiver<Contact> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    fn message_status_chan(&self) -> tokio::sync::broadcast::Receiver<SendMessageStatus> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    fn backfill_task_chan(&self) -> tokio::sync::broadcast::Receiver<BackfillTask> {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
    
    async fn get_contact_info(&self, identifier: &str) -> Result<Option<Contact>> {
        let api = self.api.read().await;
        api.get_contact_info(identifier).await
    }
    
    async fn get_contact_list(&self) -> Result<Vec<Contact>> {
        let api = self.api.read().await;
        api.get_contact_list().await
    }
    
    async fn search_contact_list(&self, search_terms: &str) -> Result<Vec<Contact>> {
        let api = self.api.read().await;
        api.search_contact_list(search_terms).await
    }
    
    async fn refresh_contact_list(&self) -> Result<()> {
        let api = self.api.read().await;
        api.refresh_contact_list().await
    }
    
    async fn get_chat_info(&self, chat_id: &str, thread_id: Option<&str>) -> Result<Option<ChatInfo>> {
        let api = self.api.read().await;
        api.get_chat_info(chat_id, thread_id).await
    }
    
    async fn get_group_avatar(&self, chat_id: &str) -> Result<Option<Attachment>> {
        let api = self.api.read().await;
        api.get_group_avatar(chat_id).await
    }
    
    async fn resolve_identifier(&self, identifier: &str) -> Result<String> {
        let api = self.api.read().await;
        api.resolve_identifier(identifier).await
    }
    
    async fn prepare_dm(&self, guid: &str) -> Result<()> {
        let api = self.api.read().await;
        api.prepare_dm(guid).await
    }
    
    async fn create_group(&self, guids: Vec<&str>) -> Result<CreateGroupResponse> {
        let api = self.api.read().await;
        api.create_group(guids).await
    }
    
    async fn send_message(
        &self,
        chat_id: &str,
        text: &str,
        reply_to: Option<&str>,
        reply_to_part: Option<i32>,
        rich_link: Option<RichLink>,
        metadata: Option<MessageMetadata>,
    ) -> Result<SendResponse> {
        let api = self.api.read().await;
        api.send_message(chat_id, text, reply_to, reply_to_part, rich_link, metadata).await
    }
    
    async fn send_file(
        &self,
        chat_id: &str,
        text: &str,
        filename: &str,
        data: Vec<u8>,
        reply_to: Option<&str>,
        reply_to_part: Option<i32>,
        mime_type: &str,
        voice_memo: bool,
        metadata: Option<MessageMetadata>,
    ) -> Result<SendResponse> {
        let api = self.api.read().await;
        api.send_file(chat_id, text, filename, data, reply_to, reply_to_part, mime_type, voice_memo, metadata).await
    }
    
    async fn send_tapback(
        &self,
        chat_id: &str,
        target_guid: &str,
        target_part: i32,
        tapback: TapbackType,
        remove: bool,
    ) -> Result<SendResponse> {
        let api = self.api.read().await;
        api.send_tapback(chat_id, target_guid, target_part, tapback, remove).await
    }
    
    async fn send_read_receipt(&self, chat_id: &str, read_up_to: &str) -> Result<()> {
        let api = self.api.read().await;
        api.send_read_receipt(chat_id, read_up_to).await
    }
    
    async fn send_typing_notification(&self, chat_id: &str, typing: bool) -> Result<()> {
        let api = self.api.read().await;
        api.send_typing_notification(chat_id, typing).await
    }
    
    async fn unsend_message(
        &self,
        chat_id: &str,
        target_guid: &str,
        target_part: i32,
    ) -> Result<SendResponse> {
        let api = self.api.read().await;
        api.unsend_message(chat_id, target_guid, target_part).await
    }
    
    async fn edit_message(
        &self,
        chat_id: &str,
        target_guid: &str,
        new_text: &str,
        target_part: i32,
    ) -> Result<SendResponse> {
        let api = self.api.read().await;
        api.edit_message(chat_id, target_guid, new_text, target_part).await
    }
    
    fn capabilities(&self) -> ConnectorCapabilities {
        // This should be async, but for now return default
        ConnectorCapabilities::default()
    }
}
