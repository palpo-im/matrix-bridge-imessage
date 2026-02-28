use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::config::PlatformConfig;

use super::super::interface::{BackfillTask, IMessageAPI};
use super::super::structs::{
    Attachment, ChatIdentifier, ChatInfo, ConnectorCapabilities, Contact, CreateGroupResponse,
    Message, MessageMetadata, ReadReceipt, RichLink, SendMessageStatus, SendResponse, TapbackType,
    TypingNotification,
};

pub struct MacNosipClient;

impl MacNosipClient {
    pub fn new(_config: &PlatformConfig) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl IMessageAPI for MacNosipClient {
    async fn start(&mut self, ready_callback: Option<Box<dyn Fn() + Send>>) -> Result<()> {
        if let Some(cb) = ready_callback {
            cb();
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn get_messages_since_date(
        &self,
        _chat_id: &str,
        _min_date: DateTime<Utc>,
        _backfill_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        Ok(Vec::new())
    }

    async fn get_messages_between(
        &self,
        _chat_id: &str,
        _min_date: DateTime<Utc>,
        _max_date: DateTime<Utc>,
    ) -> Result<Vec<Message>> {
        Ok(Vec::new())
    }

    async fn get_messages_before_with_limit(
        &self,
        _chat_id: &str,
        _before: DateTime<Utc>,
        _limit: i32,
    ) -> Result<Vec<Message>> {
        Ok(Vec::new())
    }

    async fn get_messages_with_limit(
        &self,
        _chat_id: &str,
        _limit: i32,
        _backfill_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        Ok(Vec::new())
    }

    async fn get_chats_with_messages_after(
        &self,
        _min_date: DateTime<Utc>,
    ) -> Result<Vec<ChatIdentifier>> {
        Ok(Vec::new())
    }

    async fn get_message(&self, _guid: &str) -> Result<Option<Message>> {
        Ok(None)
    }

    fn message_chan(&self) -> tokio::sync::broadcast::Receiver<Message> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    fn read_receipt_chan(&self) -> tokio::sync::broadcast::Receiver<ReadReceipt> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    fn typing_notification_chan(&self) -> tokio::sync::broadcast::Receiver<TypingNotification> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    fn chat_chan(&self) -> tokio::sync::broadcast::Receiver<ChatInfo> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    fn contact_chan(&self) -> tokio::sync::broadcast::Receiver<Contact> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    fn message_status_chan(&self) -> tokio::sync::broadcast::Receiver<SendMessageStatus> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    fn backfill_task_chan(&self) -> tokio::sync::broadcast::Receiver<BackfillTask> {
        let (_, rx) = tokio::sync::broadcast::channel(1);
        rx
    }

    async fn get_contact_info(&self, _identifier: &str) -> Result<Option<Contact>> {
        Ok(None)
    }

    async fn get_contact_list(&self) -> Result<Vec<Contact>> {
        Ok(Vec::new())
    }

    async fn search_contacts(&self, _search_terms: &str) -> Result<Vec<Contact>> {
        Ok(Vec::new())
    }

    async fn refresh_contacts(&self) -> Result<()> {
        Ok(())
    }

    async fn start_chat(&self, _identifier: &str) -> Result<ChatInfo> {
        Err(anyhow!("Not implemented"))
    }

    async fn merge_chat(&self, _chat_id: &str) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    async fn unmerge_chat(&self, _chat_id: &str) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    async fn get_chat_info(&self, _chat_id: &str, _thread_id: Option<&str>) -> Result<Option<ChatInfo>> {
        Ok(None)
    }

    async fn get_group_avatar(&self, _chat_id: &str) -> Result<Option<Attachment>> {
        Ok(None)
    }

    async fn resolve_identifier(&self, identifier: &str) -> Result<String> {
        Ok(identifier.to_owned())
    }

    async fn prepare_dm(&self, _guid: &str) -> Result<()> {
        Ok(())
    }

    async fn create_group(&self, _guids: Vec<&str>) -> Result<CreateGroupResponse> {
        Err(anyhow!("Not implemented"))
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
        Err(anyhow!("Not implemented"))
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
        Err(anyhow!("Not implemented"))
    }

    async fn send_tapback(
        &self,
        _chat_id: &str,
        _target_guid: &str,
        _target_part: i32,
        _tapback: TapbackType,
        _remove: bool,
    ) -> Result<SendResponse> {
        Err(anyhow!("Not implemented"))
    }

    async fn send_read_receipt(&self, _chat_id: &str, _read_up_to: &str) -> Result<()> {
        Ok(())
    }

    async fn send_typing_notification(&self, _chat_id: &str, _typing: bool) -> Result<()> {
        Ok(())
    }

    async fn unsend_message(
        &self,
        _chat_id: &str,
        _target_guid: &str,
        _target_part: i32,
    ) -> Result<SendResponse> {
        Err(anyhow!("Not implemented"))
    }

    async fn edit_message(
        &self,
        _chat_id: &str,
        _target_guid: &str,
        _new_text: &str,
        _target_part: i32,
    ) -> Result<SendResponse> {
        Err(anyhow!("Not implemented"))
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities::default()
    }
}
