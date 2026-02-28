pub mod bluebubbles;
pub mod client;
pub mod handler;
pub mod interface;
pub mod mac_nosip;
pub mod structs;

pub use client::IMessageClient;
pub use handler::{BackfillTask, IMessageEvent, IMessageHandler, IMessageBridge, Portal, Puppet};
pub use interface::{IMessageAPI, IMessageBridge as IMessageBridgeTrait};
pub use structs::{
    Attachment, ChatIdentifier, ChatInfo, ConnectorCapabilities, Contact, CreateGroupResponse,
    GroupActionType, Identifier, Message, MessageMetadata, ReadReceipt, RichLink,
    SendMessageStatus, SendResponse, Tapback, TapbackType, TypingNotification,
};
