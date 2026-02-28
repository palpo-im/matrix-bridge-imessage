use std::sync::Arc;

use anyhow::{Context, Result};
use matrix_bot_sdk::appservice::{Appservice, NoopAppserviceHandler};
use matrix_bot_sdk::client::{MatrixAuth, MatrixClient};
use matrix_bot_sdk::models::CreateRoom;
use serde_json::{json, Value};
use tracing::info;
use url::Url;

use crate::config::Config;

pub struct MatrixAppservice {
    config: Arc<Config>,
    pub appservice: Appservice,
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

        let registration = config.registration();
        let homeserver_url = Url::parse(&config.bridge.homeserver_url)?;
        let auth = MatrixAuth::new(&registration.as_token);
        let client = MatrixClient::new(homeserver_url, auth);

        let appservice = Appservice::new(&registration.hs_token, &registration.as_token, client)
            .with_appservice_id(&registration.id)
            .with_homeserver_name(&config.bridge.domain)
            .with_user_prefix("@_imessage_")
            .with_handler(Arc::new(NoopAppserviceHandler));

        Ok(Self { config, appservice })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn registration_preview(&self) -> Value {
        let registration = self.config.registration();
        json!({
            "id": registration.id,
            "url": registration.url,
            "sender_localpart": registration.sender_localpart,
        })
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
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered_and_joined(room_id).await?;
        intent
            .send_event(room_id, "m.room.message", &content)
            .await
            .context("failed to send message")
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

        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered_and_joined(room_id).await?;
        intent
            .send_event(room_id, "m.reaction", &content)
            .await
            .context("failed to send reaction")
    }

    pub async fn redact_event(
        &self,
        room_id: &str,
        user_id: &str,
        event_id: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered().await?;
        intent
            .redact_event(room_id, event_id, reason)
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
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered_and_joined(room_id).await?;
        intent
            .underlying_client()
            .send_read_receipt(room_id, event_id)
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
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered_and_joined(room_id).await?;
        intent
            .underlying_client()
            .set_typing(room_id, user_id, typing, timeout.map(u64::from))
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
        create_room.name = name.map(ToOwned::to_owned);
        create_room.topic = topic.map(ToOwned::to_owned);
        create_room.room_alias_name = alias.map(ToOwned::to_owned);
        if !invite.is_empty() {
            create_room.invite = invite.into_iter().map(ToOwned::to_owned).collect();
        }

        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered().await?;
        intent
            .underlying_client()
            .create_room(&create_room)
            .await
            .context("failed to create room")
    }

    pub async fn join_room(&self, room_id: &str, user_id: &str) -> Result<()> {
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent
            .join_room(room_id)
            .await
            .context("failed to join room")?;
        Ok(())
    }

    pub async fn leave_room(&self, room_id: &str, user_id: &str) -> Result<()> {
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent
            .leave_room(room_id, None)
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
        let intent = self.appservice.get_intent_for_user_id(inviter);
        intent
            .invite_user(invitee, room_id)
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
        let intent = self.appservice.get_intent_for_user_id(kicker);
        intent
            .kick_user(kickee, room_id, reason)
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
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered_and_joined(room_id).await?;
        let content = json!({ "name": name });
        intent
            .send_state_event(room_id, "m.room.name", "", &content)
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
        let intent = self.appservice.get_intent_for_user_id(user_id);
        intent.ensure_registered_and_joined(room_id).await?;
        let content = json!({ "url": avatar_url });
        intent
            .send_state_event(room_id, "m.room.avatar", "", &content)
            .await
            .context("failed to set room avatar")?;
        Ok(())
    }

    pub async fn set_user_avatar(&self, _user_id: &str, _avatar_url: &str) -> Result<()> {
        // matrix-bot-sdk currently exposes profile mutation for the authenticated user only.
        Ok(())
    }

    pub async fn set_user_displayname(&self, _user_id: &str, _displayname: &str) -> Result<()> {
        // matrix-bot-sdk currently exposes profile mutation for the authenticated user only.
        Ok(())
    }

    pub fn ghost_user_id(&self, imessage_user_guid: &str) -> String {
        ghost_user_id(imessage_user_guid, &self.config.bridge.domain)
    }

    pub fn is_namespaced_user(&self, user_id: &str) -> bool {
        is_namespaced_user(user_id)
    }
}
