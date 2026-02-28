use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::structs::{
    Attachment, ChatIdentifier, ChatInfo, ConnectorCapabilities, Contact, CreateGroupResponse,
    Message, MessageMetadata, ReadReceipt, RichLink, SendMessageStatus, SendResponse, TapbackType,
    TypingNotification,
};

#[async_trait]
pub trait IMessageAPI: Send + Sync {
    async fn start(&mut self, ready_callback: Option<Box<dyn Fn() + Send>>) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;

    async fn get_messages_since_date(
        &self,
        chat_id: &str,
        min_date: DateTime<Utc>,
        backfill_id: Option<&str>,
    ) -> Result<Vec<Message>>;

    async fn get_messages_between(
        &self,
        chat_id: &str,
        min_date: DateTime<Utc>,
        max_date: DateTime<Utc>,
    ) -> Result<Vec<Message>>;

    async fn get_messages_before_with_limit(
        &self,
        chat_id: &str,
        before: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<Message>>;

    async fn get_messages_with_limit(
        &self,
        chat_id: &str,
        limit: i32,
        backfill_id: Option<&str>,
    ) -> Result<Vec<Message>>;

    async fn get_chats_with_messages_after(
        &self,
        min_date: DateTime<Utc>,
    ) -> Result<Vec<ChatIdentifier>>;

    async fn get_message(&self, guid: &str) -> Result<Option<Message>>;

    fn message_chan(&self) -> tokio::sync::broadcast::Receiver<Message>;
    fn read_receipt_chan(&self) -> tokio::sync::broadcast::Receiver<ReadReceipt>;
    fn typing_notification_chan(&self) -> tokio::sync::broadcast::Receiver<TypingNotification>;
    fn chat_chan(&self) -> tokio::sync::broadcast::Receiver<ChatInfo>;
    fn contact_chan(&self) -> tokio::sync::broadcast::Receiver<Contact>;
    fn message_status_chan(&self) -> tokio::sync::broadcast::Receiver<SendMessageStatus>;
    fn backfill_task_chan(&self) -> tokio::sync::broadcast::Receiver<BackfillTask>;

    async fn get_contact_info(&self, identifier: &str) -> Result<Option<Contact>>;
    async fn get_contact_list(&self) -> Result<Vec<Contact>>;
    async fn search_contacts(&self, search_terms: &str) -> Result<Vec<Contact>>;
    async fn refresh_contacts(&self) -> Result<()>;

    async fn start_chat(&self, identifier: &str) -> Result<ChatInfo>;
    async fn merge_chat(&self, chat_id: &str) -> Result<()>;
    async fn unmerge_chat(&self, chat_id: &str) -> Result<()>;

    async fn get_chat_info(&self, chat_id: &str, thread_id: Option<&str>) -> Result<Option<ChatInfo>>;
    async fn get_group_avatar(&self, chat_id: &str) -> Result<Option<Attachment>>;

    async fn resolve_identifier(&self, identifier: &str) -> Result<String>;
    async fn prepare_dm(&self, guid: &str) -> Result<()>;
    async fn create_group(&self, guids: Vec<&str>) -> Result<CreateGroupResponse>;

    async fn send_message(
        &self,
        chat_id: &str,
        text: &str,
        reply_to: Option<&str>,
        reply_to_part: Option<i32>,
        rich_link: Option<RichLink>,
        metadata: Option<MessageMetadata>,
    ) -> Result<SendResponse>;

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
    ) -> Result<SendResponse>;

    async fn send_tapback(
        &self,
        chat_id: &str,
        target_guid: &str,
        target_part: i32,
        tapback: TapbackType,
        remove: bool,
    ) -> Result<SendResponse>;

    async fn send_read_receipt(&self, chat_id: &str, read_up_to: &str) -> Result<()>;
    async fn send_typing_notification(&self, chat_id: &str, typing: bool) -> Result<()>;

    async fn unsend_message(
        &self,
        chat_id: &str,
        target_guid: &str,
        target_part: i32,
    ) -> Result<SendResponse>;

    async fn edit_message(
        &self,
        chat_id: &str,
        target_guid: &str,
        new_text: &str,
        target_part: i32,
    ) -> Result<SendResponse>;

    fn capabilities(&self) -> ConnectorCapabilities;
}

pub trait IMessageBridge: Send + Sync {
    fn ping_server(&self) -> (std::time::Instant, std::time::Instant, std::time::Instant);
}

#[derive(Debug, Clone)]
pub struct BackfillTask {
    pub chat_guid: String,
    pub messages: Vec<Message>,
    pub backfill_id: String,
}
