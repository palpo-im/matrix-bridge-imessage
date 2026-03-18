pub use self::parser::{
    AuthConfig, BridgeConfig, Config, DatabaseConfig, DbType, GhostsConfig, LimitsConfig,
    LoggingConfig, LoggingFileConfig, MetricsConfig, PlatformConfig, RegistrationConfig,
    RoomConfig, UserActivityConfig,
};
pub use self::validator::ConfigError;

mod parser;
mod validator;
mod kdl_support;
