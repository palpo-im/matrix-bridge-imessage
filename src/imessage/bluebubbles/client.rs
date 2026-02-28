use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::config::PlatformConfig;

use super::interface::{BackfillTask, IMessageAPI};
use super::structs::{
    Attachment, ChatIdentifier, ChatInfo, ConnectorCapabilities, Contact, CreateGroupResponse,
    Message, MessageMetadata, ReadReceipt, RichLink, SendMessageStatus, SendResponse, TapbackType,
    TypingNotification,
};

const API_VERSION: &str = "/api/v1";

pub struct BlueBubblesClient {
    http_client: HttpClient,
    base_url: String,
    password: String,
    
    message_sender: broadcast::Sender<Message>,
    read_receipt_sender: broadcast::Sender<ReadReceipt>,
    typing_sender: broadcast::Sender<TypingNotification>,
    chat_sender: broadcast::Sender<ChatInfo>,
    contact_sender: broadcast::Sender<Contact>,
    message_status_sender: broadcast::Sender<SendMessageStatus>,
    backfill_sender: broadcast::Sender<BackfillTask>,
    
    capabilities: ConnectorCapabilities,
}

impl BlueBubblesClient {
    pub fn new(config: &PlatformConfig) -> Result<Self> {
        let base_url = config.bluebubbles_url.clone()
            .ok_or_else(|| anyhow::anyhow!("BlueBubbles URL not configured"))?;
        
        let password = config.bluebubbles_password.clone()
            .ok_or_else(|| anyhow::anyhow!("BlueBubbles password not configured"))?;
        
        let (message_sender, _) = broadcast::channel(1024);
        let (read_receipt_sender, _) = broadcast::channel(256);
        let (typing_sender, _) = broadcast::channel(256);
        let (chat_sender, _) = broadcast::channel(256);
        let (contact_sender, _) = broadcast::channel(256);
        let (message_status_sender, _) = broadcast::channel(256);
        let (backfill_sender, _) = broadcast::channel(256);
        
        let http_client = HttpClient::new();
        
        Ok(Self {
            http_client,
            base_url,
            password,
            message_sender,
            read_receipt_sender,
            typing_sender,
            chat_sender,
            contact_sender,
            message_status_sender,
            backfill_sender,
            capabilities: ConnectorCapabilities {
                message_send_responses: true,
                send_tapbacks: true,
                send_read_receipts: true,
                send_typing_notifications: true,
                send_captions: true,
                unsend_messages: true,
                edit_messages: true,
                bridge_state: true,
                message_status_checkpoints: true,
                delivered_status: true,
                contact_chat_merging: true,
                rich_links: true,
                chat_bridge_result: true,
            },
        })
    }
    
    fn build_url(&self, path: &str) -> String {
        format!("{}{}{}", self.base_url.trim_end_matches('/'), API_VERSION, path)
    }
    
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T> {
        let url = self.build_url(path);
        let response = self.http_client
            .request(method, &url)
            .header("Authorization", &self.password)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API request failed: {} - {}", status, body));
        }
        
        let result = response.json::<T>().await?;
        Ok(result)
    }
    
    async fn post<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {
        let url = self.build_url(path);
        let response = self.http_client
            .post(&url)
            .header("Authorization", &self.password)
            .json(body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API request failed: {} - {}", status, body));
        }
        
        let result = response.json::<R>().await?;
        Ok(result)
    }
}

#[async_trait]
impl IMessageAPI for BlueBubblesClient {
    async fn start(&mut self, ready_callback: Option<Box<dyn Fn() + Send>>) -> Result<()> {
        info!("Starting BlueBubbles client");
        
        // Test connection
        match self.request::<serde_json::Value>(reqwest::Method::GET, "/server/info").await {
            Ok(_) => {
                info!("Successfully connected to BlueBubbles server");
                if let Some(cb) = ready_callback {
                    cb();
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to BlueBubbles server: {}", e);
                Err(e)
            }
        }
    }
    
    async fn stop(&mut self) -> Result<()> {
        info!("Stopping BlueBubbles client");
        Ok(())
    }
    
    async fn get_messages_since_date(
        &self,
        _chat_id: &str,
        _min_date: DateTime<Utc>,
        _backfill_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn get_messages_between(
        &self,
        _chat_id: &str,
        _min_date: DateTime<Utc>,
        _max_date: DateTime<Utc>,
    ) -> Result<Vec<Message>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn get_messages_before_with_limit(
        &self,
        _chat_id: &str,
        _before: DateTime<Utc>,
        _limit: i32,
    ) -> Result<Vec<Message>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn get_messages_with_limit(
        &self,
        _chat_id: &str,
        _limit: i32,
        _backfill_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn get_chats_with_messages_after(
        &self,
        _min_date: DateTime<Utc>,
    ) -> Result<Vec<ChatIdentifier>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn get_message(&self, _guid: &str) -> Result<Option<Message>> {
        // TODO: Implement
        Ok(None)
    }
    
    fn message_chan(&self) -> tokio::sync::broadcast::Receiver<Message> {
        self.message_sender.subscribe()
    }
    
    fn read_receipt_chan(&self) -> tokio::sync::broadcast::Receiver<ReadReceipt> {
        self.read_receipt_sender.subscribe()
    }
    
    fn typing_notification_chan(&self) -> tokio::sync::broadcast::Receiver<TypingNotification> {
        self.typing_sender.subscribe()
    }
    
    fn chat_chan(&self) -> tokio::sync::broadcast::Receiver<ChatInfo> {
        self.chat_sender.subscribe()
    }
    
    fn contact_chan(&self) -> tokio::sync::broadcast::Receiver<Contact> {
        self.contact_sender.subscribe()
    }
    
    fn message_status_chan(&self) -> tokio::sync::broadcast::Receiver<SendMessageStatus> {
        self.message_status_sender.subscribe()
    }
    
    fn backfill_task_chan(&self) -> tokio::sync::broadcast::Receiver<BackfillTask> {
        self.backfill_sender.subscribe()
    }
    
    async fn get_contact_info(&self, _identifier: &str) -> Result<Option<Contact>> {
        // TODO: Implement
        Ok(None)
    }
    
    async fn get_contact_list(&self) -> Result<Vec<Contact>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn search_contact_list(&self, _search_terms: &str) -> Result<Vec<Contact>> {
        // TODO: Implement
        Ok(Vec::new())
    }
    
    async fn refresh_contact_list(&self) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
    
    async fn get_chat_info(&self, _chat_id: &str, _thread_id: Option<&str>) -> Result<Option<ChatInfo>> {
        // TODO: Implement
        Ok(None)
    }
    
    async fn get_group_avatar(&self, _chat_id: &str) -> Result<Option<Attachment>> {
        // TODO: Implement
        Ok(None)
    }
    
    async fn resolve_identifier(&self, _identifier: &str) -> Result<String> {
        // TODO: Implement
        Ok(String::new())
    }
    
    async fn prepare_dm(&self, _guid: &str) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
    
    async fn create_group(&self, _guids: Vec<&str>) -> Result<CreateGroupResponse> {
        // TODO: Implement
        Err(anyhow::anyhow!("Not implemented"))
    }
    
    async fn send_message(
        &self,
        _chat_id: &str,
        _text: &str,
        _reply_to: Option<&str>,
        _reply_to_part: Option<i32>,
        _rich_link: Option<RichLink>,
        _metadata: Option<MessageMetadata>,
    ) -> Result<SendResponse> {
        // TODO: Implement
        Err(anyhow::anyhow!("Not implemented"))
    }
    
    async fn send_file(
        &self,
        _chat_id: &str,
        _text: &str,
        _filename: &str,
        _data: Vec<u8>,
        _reply_to: Option<&str>,
        _reply_to_part: Option<i32>,
        _mime_type: &str,
        _voice_memo: bool,
        _metadata: Option<MessageMetadata>,
    ) -> Result<SendResponse> {
        // TODO: Implement
        Err(anyhow::anyhow!("Not implemented"))
    }
    
    async fn send_tapback(
        &self,
        _chat_id: &str,
        _target_guid: &str,
        _target_part: i32,
        _tapback: TapbackType,
        _remove: bool,
    ) -> Result<SendResponse> {
        // TODO: Implement
        Err(anyhow::anyhow!("Not implemented"))
    }
    
    async fn send_read_receipt(&self, _chat_id: &str, _read_up_to: &str) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
    
    async fn send_typing_notification(&self, _chat_id: &str, _typing: bool) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
    
    async fn unsend_message(
        &self,
        _chat_id: &str,
        _target_guid: &str,
        _target_part: i32,
    ) -> Result<SendResponse> {
        // TODO: Implement
        Err(anyhow::anyhow!("Not implemented"))
    }
    
    async fn edit_message(
        &self,
        _chat_id: &str,
        _target_guid: &str,
        _new_text: &str,
        _target_part: i32,
    ) -> Result<SendResponse> {
        // TODO: Implement
        Err(anyhow::anyhow!("Not implemented"))
    }
    
    fn capabilities(&self) -> ConnectorCapabilities {
        self.capabilities.clone()
    }
}
