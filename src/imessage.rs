pub mod bluebubbles;
pub mod client;
pub mod interface;
pub mod mac_nosip;
pub mod structs;

pub use client::IMessageClient;
pub use interface::{IMessageAPI, IMessageBridge};
pub use structs::{
    Attachment, ChatIdentifier, ChatInfo, ConnectorCapabilities, Contact, CreateGroupResponse,
    GroupActionType, Identifier, Message, MessageMetadata, ReadReceipt, RichLink,
    SendMessageStatus, SendResponse, Tapback, TapbackType, TypingNotification,
};
