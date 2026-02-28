use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    MissingField(String),
    PlaceholderValue(String),
    InvalidDatabaseUrl(String),
    InvalidPlatform(String),
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingField(field) => {
                write!(f, "Missing required field: {}", field)
            }
            ConfigError::PlaceholderValue(field) => {
                write!(f, "Field {} still has placeholder value", field)
            }
            ConfigError::InvalidDatabaseUrl(url) => {
                write!(f, "Invalid database URL: {}", url)
            }
            ConfigError::InvalidPlatform(platform) => {
                write!(
                    f,
                    "Invalid platform: {}. Valid options: mac, mac-nosip, bluebubbles",
                    platform
                )
            }
            ConfigError::ParseError(msg) => {
                write!(f, "Configuration parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}
