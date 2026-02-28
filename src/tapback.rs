use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::db::models::Tapback as DBTapback;
use crate::imessage::{Tapback as IMessageTapback, TapbackType};
use crate::matrix::MatrixClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionEmoji {
    Like,
    Love,
    Dislike,
    Laugh,
    Emphasize,
    Question,
    Custom,
}

impl ReactionEmoji {
    pub fn from_emoji(emoji: &str) -> Option<Self> {
        match emoji {
            "👍" | "like" => Some(Self::Like),
            "❤️" | "love" => Some(Self::Love),
            "👎" | "dislike" => Some(Self::Dislike),
            "😂" | "laugh" => Some(Self::Laugh),
            "‼️" | "emphasize" => Some(Self::Emphasize),
            "❓" | "question" => Some(Self::Question),
            _ => Some(Self::Custom),
        }
    }

    pub fn to_emoji(&self) -> &'static str {
        match self {
            Self::Like => "👍",
            Self::Love => "❤️",
            Self::Dislike => "👎",
            Self::Laugh => "😂",
            Self::Emphasize => "‼️",
            Self::Question => "❓",
            Self::Custom => "❓",
        }
    }

    pub fn to_tapback_type(&self) -> TapbackType {
        match self {
            Self::Like => TapbackType::Like,
            Self::Love => TapbackType::Love,
            Self::Dislike => TapbackType::Dislike,
            Self::Laugh => TapbackType::Laugh,
            Self::Emphasize => TapbackType::Emphasize,
            Self::Question => TapbackType::Question,
            Self::Custom => TapbackType::Question,
        }
    }

    pub fn from_tapback_type(tapback_type: TapbackType) -> Option<Self> {
        match tapback_type {
            TapbackType::Like => Some(Self::Like),
            TapbackType::Love => Some(Self::Love),
            TapbackType::Dislike => Some(Self::Dislike),
            TapbackType::Laugh => Some(Self::Laugh),
            TapbackType::Emphasize => Some(Self::Emphasize),
            TapbackType::Question => Some(Self::Question),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reaction {
    pub portal_guid: String,
    pub message_guid: String,
    pub message_part: i32,
    pub sender_guid: String,
    pub reaction_type: ReactionEmoji,
    pub matrix_event_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct TapbackManager {
    matrix: Arc<MatrixClient>,
    cache: RwLock<HashMap<String, Vec<Reaction>>>,
}

impl TapbackManager {
    pub fn new(matrix: Arc<MatrixClient>) -> Self {
        Self {
            matrix,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn handle_matrix_reaction(
        &self,
        portal_guid: &str,
        message_mxid: &str,
        reaction_mxid: &str,
        emoji: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Handling Matrix reaction {} to message {} in portal {}",
            reaction_mxid, message_mxid, portal_guid
        );

        let reaction_emoji = match ReactionEmoji::from_emoji(emoji) {
            Some(e) => e,
            None => {
                error!("Unknown reaction type: {}", emoji);
                return Err(format!("Unknown reaction type: {}", emoji).into());
            }
        };

        let target_message = match DBTapback::get_message_by_mxid(message_mxid)? {
            Some(msg) => msg,
            None => {
                error!("Unknown reaction target: {}", message_mxid);
                return Err(format!("Unknown reaction target: {}", message_mxid).into());
            }
        };

        let tapback_type = reaction_emoji.to_tapback_type();

        if let Some(existing) = DBTapback::get_by_message_and_sender(
            portal_guid,
            &target_message.guid,
            target_message.part,
            &target_message.sender_guid,
        )? {
            if existing.tapback_type == tapback_type {
                debug!("Ignoring outgoing tapback: type is same");
                return Ok(());
            }

            debug!(
                "Replacing existing tapback {} with new type {:?}",
                existing.mxid, tapback_type
            );

            // TODO: Send tapback update to iMessage
            // self.imessage.send_tapback(...).await?;

            if let Err(e) = self.matrix.redact_event(portal_guid, &existing.mxid).await {
                error!("Failed to redact old tapback {}: {}", existing.mxid, e);
            }

            existing.update(reaction_mxid.to_string(), tapback_type)?;
        } else {
            debug!("Creating new tapback");

            // TODO: Send tapback to iMessage
            // let response = self.imessage.send_tapback(...).await?;

            DBTapback::create(
                portal_guid,
                &target_message.guid,
                target_message.part,
                &target_message.sender_guid,
                tapback_type,
                reaction_mxid,
            )?;
        }

        info!("Successfully handled Matrix reaction {}", reaction_mxid);
        Ok(())
    }

    pub async fn handle_matrix_redaction(
        &self,
        portal_guid: &str,
        redaction_mxid: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Handling Matrix redaction {} in portal {}",
            redaction_mxid, portal_guid
        );

        if let Some(tapback) = DBTapback::get_by_mxid(redaction_mxid)? {
            debug!("Redacting tapback to message {}", tapback.message_guid);

            // TODO: Send tapback removal to iMessage
            // self.imessage.remove_tapback(...).await?;

            tapback.delete()?;

            info!("Successfully removed tapback {}", redaction_mxid);
            return Ok(());
        }

        debug!("No tapback found for redaction {}", redaction_mxid);
        Ok(())
    }

    pub async fn handle_imessage_tapback(
        &self,
        portal_guid: &str,
        tapback: IMessageTapback,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Handling iMessage tapback to {} in portal {}",
            tapback.message_guid, portal_guid
        );

        let reaction_emoji = match ReactionEmoji::from_tapback_type(tapback.tapback_type) {
            Some(e) => e,
            None => {
                error!("Unknown tapback type: {:?}", tapback.tapback_type);
                return Err(format!("Unknown tapback type: {:?}", tapback.tapback_type).into());
            }
        };

        if tapback.remove {
            debug!("Removing tapback from iMessage");

            if let Some(existing) = DBTapback::get_by_message_and_sender(
                portal_guid,
                &tapback.message_guid,
                tapback.message_part,
                &tapback.sender_guid,
            )? {
                if let Some(mxid) = self.matrix.get_room_id(portal_guid).await? {
                    if let Err(e) = self.matrix.redact_event(&mxid, &existing.mxid).await {
                        error!("Failed to redact Matrix tapback {}: {}", existing.mxid, e);
                    }
                }

                existing.delete()?;
            }

            info!("Successfully removed iMessage tapback");
        } else {
            debug!("Adding tapback from iMessage");

            if let Some(existing) = DBTapback::get_by_message_and_sender(
                portal_guid,
                &tapback.message_guid,
                tapback.message_part,
                &tapback.sender_guid,
            )? {
                if existing.tapback_type == tapback.tapback_type {
                    debug!("Ignoring incoming tapback: type is same");
                    return Ok(());
                }

                if let Some(mxid) = self.matrix.get_room_id(portal_guid).await? {
                    if let Err(e) = self.matrix.redact_event(&mxid, &existing.mxid).await {
                        error!("Failed to redact old tapback {}: {}", existing.mxid, e);
                    }
                }
            }

            let target_message = match DBTapback::get_message_by_guid(
                portal_guid,
                &tapback.message_guid,
                tapback.message_part,
            )? {
                Some(msg) => msg,
                None => {
                    error!(
                        "Unknown message for tapback: {}/{}",
                        tapback.message_guid, tapback.message_part
                    );
                    return Err("Unknown message for tapback".into());
                }
            };

            let emoji = reaction_emoji.to_emoji();

            if let Some(mxid) = self.matrix.get_room_id(portal_guid).await? {
                let reaction_mxid = self
                    .matrix
                    .send_reaction(&mxid, &target_message.mxid, emoji)
                    .await?;

                DBTapback::create(
                    portal_guid,
                    &tapback.message_guid,
                    tapback.message_part,
                    &tapback.sender_guid,
                    tapback.tapback_type,
                    &reaction_mxid,
                )?;
            }
        }

        info!("Successfully handled iMessage tapback");
        Ok(())
    }

    pub fn get_reactions_for_message(
        &self,
        portal_guid: &str,
        message_guid: &str,
        message_part: i32,
    ) -> Result<Vec<Reaction>, Box<dyn std::error::Error + Send + Sync>> {
        let tapbacks = DBTapback::get_for_message(portal_guid, message_guid, message_part)?;

        let reactions: Vec<Reaction> = tapbacks
            .into_iter()
            .map(|t| Reaction {
                portal_guid: t.portal_guid,
                message_guid: t.message_guid,
                message_part: t.message_part,
                sender_guid: t.sender_guid,
                reaction_type: ReactionEmoji::from_tapback_type(t.tapback_type)
                    .unwrap_or(ReactionEmoji::Custom),
                matrix_event_id: t.mxid,
                timestamp: t.timestamp,
            })
            .collect();

        Ok(reactions)
    }

    pub fn cache_reactions(&self, portal_guid: &str, reactions: Vec<Reaction>) {
        let mut cache = self.cache.write();
        cache.insert(portal_guid.to_string(), reactions);
    }

    pub fn get_cached_reactions(&self, portal_guid: &str) -> Option<Vec<Reaction>> {
        let cache = self.cache.read();
        cache.get(portal_guid).cloned()
    }

    pub fn clear_cache(&self, portal_guid: &str) {
        let mut cache = self.cache.write();
        cache.remove(portal_guid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_emoji_conversion() {
        assert_eq!(ReactionEmoji::from_emoji("👍"), Some(ReactionEmoji::Like));
        assert_eq!(ReactionEmoji::from_emoji("❤️"), Some(ReactionEmoji::Love));
        assert_eq!(ReactionEmoji::from_emoji("👎"), Some(ReactionEmoji::Dislike));
        assert_eq!(ReactionEmoji::from_emoji("😂"), Some(ReactionEmoji::Laugh));
        assert_eq!(ReactionEmoji::from_emoji("‼️"), Some(ReactionEmoji::Emphasize));
        assert_eq!(ReactionEmoji::from_emoji("❓"), Some(ReactionEmoji::Question));
    }

    #[test]
    fn test_tapback_type_conversion() {
        assert_eq!(
            ReactionEmoji::Like.to_tapback_type(),
            TapbackType::Like
        );
        assert_eq!(
            ReactionEmoji::Love.to_tapback_type(),
            TapbackType::Love
        );
    }

    #[test]
    fn test_emoji_to_string() {
        assert_eq!(ReactionEmoji::Like.to_emoji(), "👍");
        assert_eq!(ReactionEmoji::Love.to_emoji(), "❤️");
    }
}
