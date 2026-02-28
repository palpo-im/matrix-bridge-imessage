use std::sync::Arc;

use anyhow::{Context, Result};
use matrix_bot_sdk::appservice::{Appservice, AppserviceHandler};
use matrix_bot_sdk::client::{MatrixAuth, MatrixClient};
use matrix_bot_sdk::models::CreateRoom;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::config::Config;

pub mod command_handler;
pub mod event_handler;

pub use self::command_handler::{
    MatrixCommandHandler, MatrixCommandOutcome, MatrixCommandPermission,
};
pub use self::event_handler::{MatrixEventHandler, MatrixEventHandlerImpl, MatrixEventProcessor};

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

pub struct BridgeAppserviceHandler {
    processor: Option<Arc<MatrixEventProcessor>>,
}

#[async_trait::async_trait]
impl AppserviceHandler for BridgeAppserviceHandler {
    async fn on_transaction(&self, _txn_id: &str, body: &Value) -> Result<()> {
        let Some(processor) = &self.processor else {
            return Ok(());
        };

        if let Some(events) = body.get("events").and_then(|v| v.as_array()) {
            for event in events {
                let Some(room_id) = event.get("room_id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(sender) = event.get("sender").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(event_type) = event.get("type").and_then(|v| v.as_str()) else {
                    continue;
                };

                let matrix_event = MatrixEvent {
                    event_id: event
                        .get("event_id")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    event_type: event_type.to_owned(),
                    room_id: room_id.to_owned(),
                    sender: sender.to_owned(),
                    state_key: event
                        .get("state_key")
                        .and_then(|v| v.as_str())
                        .map(ToOwned::to_owned),
                    content: event.get("content").cloned(),
                    timestamp: event.get("origin_server_ts").map(|v| v.to_string()),
                };

                if let Err(e) = processor.process_event(matrix_event).await {
                    error!("error processing event: {}", e);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct MatrixAppservice {
    config: Arc<Config>,
    pub appservice: Appservice,
    handler: Arc<RwLock<BridgeAppserviceHandler>>,
}

#[derive(Debug, Clone)]
pub struct MatrixEvent {
    pub event_id: Option<String>,
    pub event_type: String,
    pub room_id: String,
    pub sender: String,
    pub state_key: Option<String>,
    pub content: Option<Value>,
    pub timestamp: Option<String>,
}

fn build_matrix_message_content(
    body: &str,
    reply_to: Option<&str>,
    edit_of: Option<&str>,
) -> Value {
    let mut content = json!({
        "msgtype": "m.text",
        "body": body,
    });

    if let Some(reply_id) = reply_to {
        content["m.relates_to"] = json!({
            "m.in_reply_to": {
                "event_id": reply_id
            }
        });
    }

    if let Some(edit_event_id) = edit_of {
        content["m.new_content"] = json!({
            "msgtype": "m.text",
            "body": body,
        });
        content["m.relates_to"] = json!({
            "rel_type": "m.replace",
            "event_id": edit_event_id,
        });
        content["body"] = format!("* {body}").into();
    }

    content
}

fn ghost_user_id(imessage_user_guid: &str, domain: &str) -> String {
    let clean_guid = imessage_user_guid
        .replace([';', ':', '/', '@'], "_")
        .replace(' ', "_");
    format!("@_imessage_{}:{}", clean_guid, domain)
}

fn is_namespaced_user(user_id: &str) -> bool {
    user_id.starts_with("@_imessage_")
}

impl MatrixAppservice {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!(
            "initializing matrix appservice for {}",
            config.bridge.domain
        );

        let homeserver_url = Url::parse(&config.bridge.homeserver_url)?;
        let auth = MatrixAuth::new(&config.registration.appservice_token);
        let client = MatrixClient::new(homeserver_url, auth);

        let handler = Arc::new(RwLock::new(BridgeAppserviceHandler { processor: None }));

        struct HandlerWrapper(Arc<RwLock<BridgeAppserviceHandler>>);
        #[async_trait::async_trait]
        impl AppserviceHandler for HandlerWrapper {
            async fn on_transaction(&self, txn_id: &str, body: &Value) -> Result<()> {
                self.0.read().await.on_transaction(txn_id, body).await
            }
        }

        let registration = config.registration.clone();
        let appservice = Appservice::with_handler(
            client,
            &registration.id,
            &registration.as_token,
            HandlerWrapper(handler.clone()),
        )
        .await?;

        Ok(Self {
            config,
            appservice,
            handler,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn set_processor(&self, processor: Arc<MatrixEventProcessor>) {
        let mut handler = self.handler.write().await;
        handler.processor = Some(processor);
    }

    pub async fn send_message(
        &self,
        room_id: &str,
        user_id: &str,
        body: &str,
        reply_to: Option<&str>,
        edit_of: Option<&str>,
    ) -> Result<String> {
        let content = build_matrix_message_content(body, reply_to, edit_of);
        
        let event_id = self.appservice
            .client()
            .send_message(room_id, user_id, content)
            .await
            .context("failed to send message")?;
        
        Ok(event_id)
    }

    pub async fn send_reaction(
        &self,
        room_id: &str,
        user_id: &str,
        event_id: &str,
        key: &str,
    ) -> Result<String> {
        let content = json!({
            "m.relates_to": {
                "rel_type": "m.annotation",
                "event_id": event_id,
                "key": key
            }
        });

        let reaction_event_id = self.appservice
            .client()
            .send_state_event(room_id, user_id, "m.reaction", None, content)
            .await
            .context("failed to send reaction")?;

        Ok(reaction_event_id)
    }

    pub async fn redact_event(
        &self,
        room_id: &str,
        user_id: &str,
        event_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        self.appservice
            .client()
            .redact(room_id, user_id, event_id, reason)
            .await
            .context("failed to redact event")?;

        Ok(())
    }

    pub async fn send_read_receipt(
        &self,
        room_id: &str,
        user_id: &str,
        event_id: &str,
    ) -> Result<()> {
        self.appservice
            .client()
            .send_read_receipt(room_id, user_id, event_id)
            .await
            .context("failed to send read receipt")?;

        Ok(())
    }

    pub async fn send_typing_notification(
        &self,
        room_id: &str,
        user_id: &str,
        typing: bool,
        timeout: Option<u32>,
    ) -> Result<()> {
        self.appservice
            .client()
            .send_typing(room_id, user_id, typing, timeout)
            .await
            .context("failed to send typing notification")?;

        Ok(())
    }

    pub async fn create_room(
        &self,
        user_id: &str,
        name: Option<&str>,
        topic: Option<&str>,
        alias: Option<&str>,
        invite: Vec<&str>,
    ) -> Result<String> {
        let mut create_room = CreateRoom::default();
        
        if let Some(name) = name {
            create_room.name = Some(name.to_string());
        }
        
        if let Some(topic) = topic {
            create_room.topic = Some(topic.to_string());
        }
        
        if let Some(alias) = alias {
            create_room.room_alias_name = Some(alias.to_string());
        }
        
        if !invite.is_empty() {
            create_room.invite = invite.iter().map(|s| s.to_string()).collect();
        }

        let room_id = self.appservice
            .client()
            .create_room(user_id, create_room)
            .await
            .context("failed to create room")?;

        Ok(room_id)
    }

    pub async fn join_room(&self, room_id: &str, user_id: &str) -> Result<()> {
        self.appservice
            .client()
            .join_room(room_id, user_id)
            .await
            .context("failed to join room")?;

        Ok(())
    }

    pub async fn leave_room(&self, room_id: &str, user_id: &str) -> Result<()> {
        self.appservice
            .client()
            .leave_room(room_id, user_id)
            .await
            .context("failed to leave room")?;

        Ok(())
    }

    pub async fn invite_user(
        &self,
        room_id: &str,
        inviter: &str,
        invitee: &str,
    ) -> Result<()> {
        self.appservice
            .client()
            .invite_user(room_id, inviter, invitee)
            .await
            .context("failed to invite user")?;

        Ok(())
    }

    pub async fn kick_user(
        &self,
        room_id: &str,
        kicker: &str,
        kickee: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        self.appservice
            .client()
            .kick_user(room_id, kicker, kickee, reason)
            .await
            .context("failed to kick user")?;

        Ok(())
    }

    pub async fn set_room_name(
        &self,
        room_id: &str,
        user_id: &str,
        name: &str,
    ) -> Result<()> {
        let content = json!({
            "name": name
        });

        self.appservice
            .client()
            .send_state_event(room_id, user_id, "m.room.name", None, content)
            .await
            .context("failed to set room name")?;

        Ok(())
    }

    pub async fn set_room_avatar(
        &self,
        room_id: &str,
        user_id: &str,
        avatar_url: &str,
    ) -> Result<()> {
        let content = json!({
            "url": avatar_url
        });

        self.appservice
            .client()
            .send_state_event(room_id, user_id, "m.room.avatar", None, content)
            .await
            .context("failed to set room avatar")?;

        Ok(())
    }

    pub async fn set_user_avatar(
        &self,
        user_id: &str,
        avatar_url: &str,
    ) -> Result<()> {
        let content = json!({
            "avatar_url": avatar_url
        });

        self.appservice
            .client()
            .send_state_event("", user_id, "m.room.member", Some(user_id), content)
            .await
            .context("failed to set user avatar")?;

        Ok(())
    }

    pub async fn set_user_displayname(
        &self,
        user_id: &str,
        displayname: &str,
    ) -> Result<()> {
        let content = json!({
            "displayname": displayname
        });

        self.appservice
            .client()
            .send_state_event("", user_id, "m.room.member", Some(user_id), content)
            .await
            .context("failed to set user displayname")?;

        Ok(())
    }

    pub fn ghost_user_id(&self, imessage_user_guid: &str) -> String {
        ghost_user_id(imessage_user_guid, &self.config.bridge.domain)
    }

    pub fn is_namespaced_user(&self, user_id: &str) -> bool {
        is_namespaced_user(user_id)
    }
}
