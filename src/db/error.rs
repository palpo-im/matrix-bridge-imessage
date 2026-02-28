use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    QueryError(String),
    MigrationError(String),
    NotFound(String),
    InvalidConfig(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionError(msg) => write!(f, "Database connection error: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "Database query error: {}", msg),
            DatabaseError::MigrationError(msg) => write!(f, "Database migration error: {}", msg),
            DatabaseError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DatabaseError::InvalidConfig(msg) => write!(f, "Invalid database config: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}
