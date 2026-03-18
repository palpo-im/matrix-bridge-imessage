use std::path::Path;

use anyhow::{Context, Result};
use config::{Config as ConfigLib, File, FileFormat};
use serde::Deserialize;
use std::fs;

use super::validator::ConfigError;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bridge: BridgeConfig,
    pub platform: PlatformConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub room: RoomConfig,
    pub limits: LimitsConfig,
    pub ghosts: GhostsConfig,
    pub metrics: MetricsConfig,
}

impl Config {
    pub fn registration(&self) -> RegistrationConfig {
        RegistrationConfig {
            id: format!("imessage-{}", self.bridge.bridge_id),
            url: format!("http://{}:{}", self.bridge.bind_address, self.bridge.port),
            as_token: self.bridge.appservice_token.clone(),
            hs_token: self.bridge.homeserver_token.clone(),
            sender_localpart: format!("_imessage_{}", self.bridge.bridge_id),
            rate_limited: false,
            protocols: vec!["m.bridge.imessage".to_string()],
            namespaces: NamespacesConfig {
                users: vec![NamespaceConfig {
                    exclusive: true,
                    regex: format!("@_imessage_.*:{}", self.bridge.domain),
                }],
                aliases: vec![],
                rooms: vec![],
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeConfig {
    pub domain: String,
    pub homeserver_url: String,
    pub port: u16,
    pub bind_address: String,
    pub bridge_id: String,
    pub appservice_token: String,
    pub homeserver_token: String,
    pub presence_interval: u64,
    pub disable_presence: bool,
    pub disable_typing_notifications: bool,
    pub disable_deletion_forwarding: bool,
    pub disable_portal_bridging: bool,
    pub enable_self_service_bridging: bool,
    pub disable_read_receipts: bool,
    pub disable_join_leave_notifications: bool,
    pub disable_invite_notifications: bool,
    pub disable_room_topic_notifications: bool,
    pub admin_mxid: String,
    pub user_limit: Option<u32>,
    pub user_activity: UserActivityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlatformConfig {
    pub platform: String,
    pub bluebubbles_url: Option<String>,
    pub bluebubbles_password: Option<String>,
    pub imessage_rest_path: Option<String>,
    pub imessage_rest_args: Option<Vec<String>>,
    pub contacts_mode: Option<String>,
    pub hacky_set_locale: Option<String>,
    pub environment: Option<Vec<String>>,
    pub log_ipc_payloads: Option<bool>,
    pub unix_socket: Option<String>,
    pub ping_interval: Option<u64>,
    pub delete_media_after_upload: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub shared_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub line_date_format: String,
    pub format: String,
    pub files: Vec<LoggingFileConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingFileConfig {
    pub path: String,
    pub max_size: Option<u64>,
    pub max_files: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub filename: Option<String>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum DbType {
    Sqlite,
    Postgres,
    Mysql,
}

impl DatabaseConfig {
    pub fn db_type(&self) -> Result<DbType, ConfigError> {
        if self.filename.is_some() {
            return Ok(DbType::Sqlite);
        }

        if self.url.starts_with("sqlite://") || self.url.starts_with("sqlite:") {
            return Ok(DbType::Sqlite);
        }

        if self.url.starts_with("postgresql://") || self.url.starts_with("postgres://") {
            return Ok(DbType::Postgres);
        }

        if self.url.starts_with("mysql://") {
            return Ok(DbType::Mysql);
        }

        Err(ConfigError::InvalidDatabaseUrl(self.url.clone()))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoomConfig {
    pub default_visibility: String,
    pub room_alias_prefix: String,
    pub enable_room_creation: bool,
    pub kick_for: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitsConfig {
    pub room_ghost_join_delay: u64,
    pub imessage_send_delay: u64,
    pub room_count: i32,
    pub matrix_event_age_limit_ms: u64,
    pub max_message_age_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhostsConfig {
    pub nick_pattern: String,
    pub username_pattern: String,
    pub username_template: String,
    pub displayname_template: String,
    pub avatar_url_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserActivityConfig {
    pub min_user_active_days: u32,
    pub inactive_after_days: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationConfig {
    pub id: String,
    pub url: String,
    pub as_token: String,
    pub hs_token: String,
    pub sender_localpart: String,
    pub rate_limited: bool,
    pub protocols: Vec<String>,
    pub namespaces: NamespacesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespacesConfig {
    pub users: Vec<NamespaceConfig>,
    pub aliases: Vec<NamespaceConfig>,
    pub rooms: Vec<NamespaceConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamespaceConfig {
    pub exclusive: bool,
    pub regex: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let args = crate::cli::parse_args();
        let path = Path::new(&args.config);
        if !path.exists() {
            let kdl_path = Path::new("config.kdl");
            if kdl_path.exists() {
                return Self::load_from_path(kdl_path);
            }
        }
        Self::load_from_path(&args.config)
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if super::kdl_support::is_kdl_file(path) {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config from {:?}", path))?;
            let config: Config = super::kdl_support::parse_kdl_config(&content)
                .map_err(ConfigError::Kdl)?;
            config.validate()?;
            return Ok(config);
        }

        let config = ConfigLib::builder()
            .add_source(File::from(path).format(FileFormat::Yaml))
            .build()
            .with_context(|| format!("Failed to load config from {:?}", path))?;

        let config: Config = config
            .try_deserialize()
            .with_context(|| "Failed to deserialize config")?;

        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate bridge config
        if self.bridge.domain.is_empty() {
            return Err(ConfigError::MissingField("bridge.domain".to_string()));
        }

        if self.bridge.homeserver_url.is_empty() {
            return Err(ConfigError::MissingField(
                "bridge.homeserver_url".to_string(),
            ));
        }

        if self.bridge.appservice_token == "CHANGE_ME_AS_TOKEN" {
            return Err(ConfigError::PlaceholderValue(
                "bridge.appservice_token".to_string(),
            ));
        }

        if self.bridge.homeserver_token == "CHANGE_ME_HS_TOKEN" {
            return Err(ConfigError::PlaceholderValue(
                "bridge.homeserver_token".to_string(),
            ));
        }

        // Validate platform config
        let valid_platforms = ["mac", "mac-nosip", "bluebubbles"];
        if !valid_platforms.contains(&self.platform.platform.as_str()) {
            return Err(ConfigError::InvalidPlatform(self.platform.platform.clone()));
        }

        // Validate BlueBubbles config
        if self.platform.platform == "bluebubbles" {
            if self.platform.bluebubbles_url.is_none() {
                return Err(ConfigError::MissingField(
                    "platform.bluebubbles_url".to_string(),
                ));
            }
            if self.platform.bluebubbles_password.is_none() {
                return Err(ConfigError::MissingField(
                    "platform.bluebubbles_password".to_string(),
                ));
            }
        }

        // Validate database config
        self.database.db_type()?;

        Ok(())
    }
}
