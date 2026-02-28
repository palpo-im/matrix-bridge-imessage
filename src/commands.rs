use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, error, info};

use crate::imessage::IMessageAPI;
use crate::matrix::MatrixClient;

pub struct CommandContext {
    pub sender: String,
    pub room_id: String,
    pub args: Vec<String>,
    pub raw_args: String,
    pub reply_to: Option<String>,
}

impl CommandContext {
    pub fn new(sender: String, room_id: String, args: Vec<String>, raw_args: String) -> Self {
        Self {
            sender,
            room_id,
            args,
            raw_args,
            reply_to: None,
        }
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    
    async fn execute(
        &self,
        ctx: CommandContext,
        matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct PMCommand;

#[async_trait]
impl Command for PMCommand {
    fn name(&self) -> &str {
        "pm"
    }

    fn description(&self) -> &str {
        "Creates a new PM with the specified number or address"
    }

    fn usage(&self) -> &str {
        "pm <international phone number> OR pm <apple id email address>"
    }

    async fn execute(
        &self,
        ctx: CommandContext,
        matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing PM command with args: {:?}", ctx.args);

        if ctx.args.is_empty() {
            return Ok(format!("**Usage:** `{}`", self.usage()));
        }

        let identifier = &ctx.args[0];
        
        match imessage.start_chat(identifier).await {
            Ok(chat_info) => {
                let room_id = matrix.create_room(&chat_info).await?;
                
                info!("Created PM room {} for {}", room_id, identifier);
                
                Ok(format!(
                    "Created portal room [{}](https://matrix.to/#/{}) and invited you to it.",
                    room_id, room_id
                ))
            }
            Err(e) => {
                error!("Failed to start PM: {}", e);
                Ok(format!("Failed to start PM: {}", e))
            }
        }
    }
}

pub struct SearchContactsCommand;

#[async_trait]
impl Command for SearchContactsCommand {
    fn name(&self) -> &str {
        "search-contacts"
    }

    fn description(&self) -> &str {
        "Searches contacts based on name, phone, and email (only for BlueBubbles mode)"
    }

    fn usage(&self) -> &str {
        "search-contacts <search terms>"
    }

    async fn execute(
        &self,
        ctx: CommandContext,
        matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing search-contacts command with args: {:?}", ctx.args);

        if ctx.args.is_empty() {
            return Ok(format!("**Usage:** `{}`", self.usage()));
        }

        match imessage.search_contacts(&ctx.raw_args).await {
            Ok(contacts) => {
                if contacts.is_empty() {
                    Ok(format!("No contacts found for search `{}`", ctx.raw_args))
                } else {
                    let mut reply = format!("Found {} contacts:\n", contacts.len());
                    
                    for contact in contacts {
                        reply.push_str(&build_contact_string(&contact));
                        reply.push_str(&"-".repeat(40));
                        reply.push('\n');
                    }
                    
                    Ok(reply)
                }
            }
            Err(e) => {
                error!("Failed to search contacts: {}", e);
                Ok(format!("Failed to search contacts: {}", e))
            }
        }
    }
}

pub struct RefreshContactsCommand;

#[async_trait]
impl Command for RefreshContactsCommand {
    fn name(&self) -> &str {
        "refresh-contacts"
    }

    fn description(&self) -> &str {
        "Request that the bridge reload cached contacts (only for BlueBubbles mode)"
    }

    fn usage(&self) -> &str {
        "refresh-contacts"
    }

    async fn execute(
        &self,
        ctx: CommandContext,
        _matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing refresh-contacts command");

        match imessage.refresh_contacts().await {
            Ok(_) => {
                info!("Contacts list refreshed");
                Ok("Contacts List updated!".to_string())
            }
            Err(e) => {
                error!("Failed to refresh contacts: {}", e);
                Ok(format!("Failed to refresh contacts: {}", e))
            }
        }
    }
}

pub struct MergeChatCommand;

#[async_trait]
impl Command for MergeChatCommand {
    fn name(&self) -> &str {
        "merge"
    }

    fn description(&self) -> &str {
        "Merge chats based on contact information"
    }

    fn usage(&self) -> &str {
        "merge <chat-id>"
    }

    async fn execute(
        &self,
        ctx: CommandContext,
        _matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing merge command with args: {:?}", ctx.args);

        if ctx.args.is_empty() {
            return Ok(format!("**Usage:** `{}`", self.usage()));
        }

        let chat_id = &ctx.args[0];
        
        match imessage.merge_chat(chat_id).await {
            Ok(_) => {
                info!("Successfully merged chat {}", chat_id);
                Ok(format!("Successfully merged chat {}", chat_id))
            }
            Err(e) => {
                error!("Failed to merge chat: {}", e);
                Ok(format!("Failed to merge chat: {}", e))
            }
        }
    }
}

pub struct UnmergeChatCommand;

#[async_trait]
impl Command for UnmergeChatCommand {
    fn name(&self) -> &str {
        "unmerge"
    }

    fn description(&self) -> &str {
        "Unmerge a previously merged chat"
    }

    fn usage(&self) -> &str {
        "unmerge <chat-id>"
    }

    async fn execute(
        &self,
        ctx: CommandContext,
        _matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Executing unmerge command with args: {:?}", ctx.args);

        if ctx.args.is_empty() {
            return Ok(format!("**Usage:** `{}`", self.usage()));
        }

        let chat_id = &ctx.args[0];
        
        match imessage.unmerge_chat(chat_id).await {
            Ok(_) => {
                info!("Successfully unmerged chat {}", chat_id);
                Ok(format!("Successfully unmerged chat {}", chat_id))
            }
            Err(e) => {
                error!("Failed to unmerge chat: {}", e);
                Ok(format!("Failed to unmerge chat: {}", e))
            }
        }
    }
}

fn build_contact_string(contact: &crate::imessage::Contact) -> String {
    let name = if contact.nickname.is_empty() {
        format!("{} {}", contact.first_name, contact.last_name).trim().to_string()
    } else {
        contact.nickname.clone()
    };

    let mut contact_info = format!("**{}**\n", name);

    if !contact.phones.is_empty() {
        contact_info.push_str("- **Phones:**\n");
        for phone in &contact.phones {
            contact_info.push_str(&format!("  - {}\n", phone));
        }
    }

    if !contact.emails.is_empty() {
        contact_info.push_str("- **Emails:**\n");
        for email in &contact.emails {
            contact_info.push_str(&format!("  - {}\n", email));
        }
    }

    contact_info
}

pub struct CommandProcessor {
    commands: Vec<Arc<dyn Command>>,
    matrix: Arc<MatrixClient>,
    imessage: Arc<dyn IMessageAPI + Send + Sync>,
}

impl CommandProcessor {
    pub fn new(
        matrix: Arc<MatrixClient>,
        imessage: Arc<dyn IMessageAPI + Send + Sync>,
    ) -> Self {
        let mut commands: Vec<Arc<dyn Command>> = vec![
            Arc::new(PMCommand),
            Arc::new(SearchContactsCommand),
            Arc::new(RefreshContactsCommand),
            Arc::new(MergeChatCommand),
            Arc::new(UnmergeChatCommand),
        ];

        Self {
            commands,
            matrix,
            imessage,
        }
    }

    pub async fn process_command(
        &self,
        command_text: &str,
        sender: String,
        room_id: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = command_text.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok("No command provided".to_string());
        }

        let command_name = parts[0].trim_start_matches('!');
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        let raw_args = parts[1..].join(" ");

        debug!("Processing command: {} with args: {:?}", command_name, args);

        let ctx = CommandContext::new(sender, room_id, args, raw_args);

        for cmd in &self.commands {
            if cmd.name() == command_name {
                return cmd.execute(ctx, self.matrix.clone(), self.imessage.clone()).await;
            }
        }

        Ok(format!("Unknown command: {}. Type `!help` for available commands.", command_name))
    }

    pub fn get_help_text(&self) -> String {
        let mut help = "Available commands:\n\n".to_string();
        
        for cmd in &self.commands {
            help.push_str(&format!("**!{}** - {}\n", cmd.name(), cmd.description()));
            help.push_str(&format!("  Usage: `{}`\n\n", cmd.usage()));
        }
        
        help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_context_creation() {
        let ctx = CommandContext::new(
            "user@example.com".to_string(),
            "room123".to_string(),
            vec!["arg1".to_string(), "arg2".to_string()],
            "arg1 arg2".to_string(),
        );
        
        assert_eq!(ctx.sender, "user@example.com");
        assert_eq!(ctx.room_id, "room123");
        assert_eq!(ctx.args.len(), 2);
    }

    #[test]
    fn test_pm_command_usage() {
        let cmd = PMCommand;
        assert_eq!(cmd.name(), "pm");
        assert!(!cmd.description().is_empty());
        assert!(!cmd.usage().is_empty());
    }
}
