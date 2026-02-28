use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(skip)]
    pub row_id: Option<i32>,

    pub guid: String,
    #[serde(skip)]
    pub time: DateTime<Utc>,
    #[serde(rename = "timestamp")]
    pub json_unix_time: f64,
    pub subject: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "chat_guid")]
    pub chat_guid: String,
    #[serde(rename = "sender_guid")]
    pub json_sender_guid: String,
    #[serde(skip)]
    pub sender: Identifier,
    #[serde(rename = "target_guid")]
    pub json_target_guid: Option<String>,
    #[serde(skip)]
    pub target: Option<Identifier>,

    pub service: String,

    #[serde(rename = "is_from_me")]
    pub is_from_me: bool,
    #[serde(rename = "is_read")]
    pub is_read: bool,
    #[serde(skip)]
    pub read_at: Option<DateTime<Utc>>,
    #[serde(rename = "read_at")]
    pub json_unix_read_at: Option<f64>,

    pub is_delivered: bool,
    pub is_sent: bool,
    pub is_emote: bool,
    #[serde(rename = "is_audio_message")]
    pub is_audio_message: bool,
    pub is_edited: bool,
    pub is_retracted: bool,

    #[serde(rename = "thread_originator_guid")]
    pub reply_to_guid: Option<String>,
    #[serde(rename = "thread_originator_part")]
    pub reply_to_part: Option<i32>,

    #[serde(rename = "associated_message")]
    pub tapback: Option<Tapback>,

    #[serde(skip)]
    pub reply_processed: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Attachment>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_notice: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<ItemType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_action_type: Option<GroupActionType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "new_group_title")]
    pub new_group_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "rich_link")]
    pub rich_link: Option<RichLink>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MessageMetadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

impl Message {
    pub fn sender_text(&self) -> String {
        if self.is_from_me {
            "self".to_string()
        } else {
            self.sender.local_id.clone()
        }
    }
}

pub type MessageMetadata = serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadReceipt {
    #[serde(rename = "sender_guid")]
    pub sender_guid: String,
    #[serde(rename = "is_from_me")]
    pub is_from_me: bool,
    #[serde(rename = "chat_guid")]
    pub chat_guid: String,
    #[serde(rename = "read_up_to")]
    pub read_up_to: String,

    #[serde(skip)]
    pub read_at: DateTime<Utc>,
    #[serde(rename = "read_at")]
    pub json_unix_read_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingNotification {
    #[serde(rename = "chat_guid")]
    pub chat_guid: String,
    pub typing: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GroupActionType {
    AddUser = 0,
    RemoveUser = 1,
    SetAvatar = 1,
    RemoveAvatar = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Message = 0,
    Member = 1,
    Name = 2,
    Avatar = 3,
    Error = -100,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phones: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "user_guid")]
    pub user_guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_identifier: Option<String>,
}

impl Contact {
    pub fn has_name(&self) -> bool {
        self.first_name.is_some() || self.last_name.is_some() || self.nickname.is_some()
    }

    pub fn name(&self) -> String {
        if let Some(ref first) = self.first_name {
            if let Some(ref last) = self.last_name {
                return format!("{} {}", first, last);
            } else {
                return first.clone();
            }
        } else if let Some(ref last) = self.last_name {
            return last.clone();
        } else if let Some(ref nick) = self.nickname {
            return nick.clone();
        } else if let Some(ref emails) = self.emails {
            if !emails.is_empty() {
                return emails[0].clone();
            }
        } else if let Some(ref phones) = self.phones {
            if !phones.is_empty() {
                return phones[0].clone();
            }
        }
        String::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    #[serde(rename = "path_on_disk")]
    pub path_on_disk: String,
    #[serde(rename = "file_name")]
    pub file_name: String,
    #[serde(rename = "mime_type")]
    pub mime_type: Option<String>,

    #[serde(skip)]
    tried_magic: bool,
}

impl Attachment {
    pub fn get_mime_type(&mut self) -> Option<String> {
        if self.mime_type.is_none() && !self.tried_magic {
            self.tried_magic = true;
            if let Ok(mime) = self.detect_mime_from_file() {
                self.mime_type = Some(mime);
            }
        }
        self.mime_type.clone()
    }

    fn detect_mime_from_file(&self) -> Result<String, std::io::Error> {
        // TODO: Implement MIME detection using file magic
        Ok("application/octet-stream".to_string())
    }

    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }

    pub fn read(&self) -> Result<Vec<u8>, std::io::Error> {
        let path = if self.path_on_disk.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}{}", home, &self.path_on_disk[1..])
        } else {
            self.path_on_disk.clone()
        };
        std::fs::read(path)
    }

    pub fn delete(&self) -> Result<(), std::io::Error> {
        std::fs::remove_file(&self.path_on_disk)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIdentifier {
    #[serde(rename = "chat_guid")]
    pub chat_guid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInfo {
    #[serde(rename = "chat_guid")]
    pub json_chat_guid: String,
    #[serde(skip)]
    pub identifier: Identifier,
    pub title: String,
    pub members: Vec<String>,
    #[serde(rename = "no_create_room")]
    pub no_create_room: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub local_id: String,
    pub service: String,
    pub is_group: bool,
}

impl Identifier {
    pub fn parse(guid: &str) -> Option<Self> {
        if guid.is_empty() {
            return None;
        }

        let parts: Vec<&str> = guid.split(';').collect();
        if parts.len() >= 3 {
            Some(Self {
                service: parts[0].to_string(),
                is_group: parts[1] == "+",
                local_id: parts[2].to_string(),
            })
        } else {
            None
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.local_id.is_empty() {
            return Ok(());
        }
        let type_char = if self.is_group { '+' } else { '-' };
        write!(f, "{};{};{}", self.service, type_char, self.local_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResponse {
    pub guid: String,
    pub service: String,
    #[serde(rename = "chat_guid")]
    pub chat_guid: String,
    #[serde(skip)]
    pub time: DateTime<Utc>,
    #[serde(rename = "timestamp")]
    pub unix_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupResponse {
    pub guid: String,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorCapabilities {
    pub message_send_responses: bool,
    pub send_tapbacks: bool,
    pub send_read_receipts: bool,
    pub send_typing_notifications: bool,
    pub send_captions: bool,
    pub unsend_messages: bool,
    pub edit_messages: bool,
    pub bridge_state: bool,
    pub message_status_checkpoints: bool,
    pub delivered_status: bool,
    pub contact_chat_merging: bool,
    pub rich_links: bool,
    pub chat_bridge_result: bool,
}

impl Default for ConnectorCapabilities {
    fn default() -> Self {
        Self {
            message_send_responses: false,
            send_tapbacks: false,
            send_read_receipts: false,
            send_typing_notifications: false,
            send_captions: false,
            unsend_messages: false,
            edit_messages: false,
            bridge_state: false,
            message_status_checkpoints: false,
            delivered_status: false,
            contact_chat_merging: false,
            rich_links: false,
            chat_bridge_result: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageStatus {
    pub guid: String,
    #[serde(rename = "chat_guid")]
    pub chat_guid: String,
    pub status: String,
    pub service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tapback {
    #[serde(rename = "associated_message_type")]
    pub tapback_type: TapbackType,
    #[serde(rename = "associated_message_guid")]
    pub target_guid: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TapbackType {
    Love = 0,
    Like = 1,
    Dislike = 2,
    Laugh = 3,
    Emphasize = 4,
    Question = 5,
}

impl TapbackType {
    pub fn from_int(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Love),
            1 => Some(Self::Like),
            2 => Some(Self::Dislike),
            3 => Some(Self::Laugh),
            4 => Some(Self::Emphasize),
            5 => Some(Self::Question),
            _ => None,
        }
    }

    pub fn to_reaction(&self) -> &'static str {
        match self {
            Self::Love => "❤️",
            Self::Like => "👍",
            Self::Dislike => "👎",
            Self::Laugh => "😂",
            Self::Emphasize => "‼️",
            Self::Question => "❓",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichLink {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<RichLinkAsset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<RichLinkVideoAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichLinkAsset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<RichLinkAssetSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<RichLinkAssetSize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichLinkAssetSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichLinkAssetSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichLinkVideoAsset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub you_tube_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming_url: Option<String>,
    pub asset: RichLinkAsset,
}
