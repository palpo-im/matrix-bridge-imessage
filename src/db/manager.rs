use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Connection, SqliteConnection};
use tracing::info;

use super::error::DatabaseError;
use super::models::{MessageMapping, NewMessageMapping, NewRoomMapping, NewUserMapping, RoomMapping, UserMapping};
use crate::config::{Config, DatabaseConfig, DbType};

#[async_trait]
pub trait Database: Send + Sync {
    async fn migrate(&self) -> Result<(), DatabaseError>;
    
    async fn get_message_mapping(&self, imessage_guid: &str) -> Result<Option<MessageMapping>, DatabaseError>;
    async fn insert_message_mapping(&self, mapping: NewMessageMapping) -> Result<MessageMapping, DatabaseError>;
    async fn delete_message_mapping(&self, imessage_guid: &str) -> Result<(), DatabaseError>;
    
    async fn get_room_mapping(&self, imessage_chat_guid: &str) -> Result<Option<RoomMapping>, DatabaseError>;
    async fn insert_room_mapping(&self, mapping: NewRoomMapping) -> Result<RoomMapping, DatabaseError>;
    async fn delete_room_mapping(&self, imessage_chat_guid: &str) -> Result<(), DatabaseError>;
    
    async fn get_user_mapping(&self, imessage_user_guid: &str) -> Result<Option<UserMapping>, DatabaseError>;
    async fn insert_user_mapping(&self, mapping: NewUserMapping) -> Result<UserMapping, DatabaseError>;
    async fn delete_user_mapping(&self, imessage_user_guid: &str) -> Result<(), DatabaseError>;
}

pub struct DatabaseManager {
    db: Arc<Box<dyn Database>>,
}

impl DatabaseManager {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let db: Box<dyn Database> = match config.db_type().map_err(|e| DatabaseError::InvalidConfig(e.to_string()))? {
            DbType::Sqlite => Box::new(SqliteDatabase::new(config)?),
            DbType::Postgres => Box::new(PostgresDatabase::new(config)?),
            DbType::Mysql => Box::new(MysqlDatabase::new(config)?),
        };
        
        Ok(Self {
            db: Arc::new(db),
        })
    }

    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        self.db.migrate().await
    }
}

#[async_trait]
impl Database for DatabaseManager {
    async fn migrate(&self) -> Result<(), DatabaseError> {
        self.db.migrate().await
    }
    
    async fn get_message_mapping(&self, imessage_guid: &str) -> Result<Option<MessageMapping>, DatabaseError> {
        self.db.get_message_mapping(imessage_guid).await
    }
    
    async fn insert_message_mapping(&self, mapping: NewMessageMapping) -> Result<MessageMapping, DatabaseError> {
        self.db.insert_message_mapping(mapping).await
    }
    
    async fn delete_message_mapping(&self, imessage_guid: &str) -> Result<(), DatabaseError> {
        self.db.delete_message_mapping(imessage_guid).await
    }
    
    async fn get_room_mapping(&self, imessage_chat_guid: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        self.db.get_room_mapping(imessage_chat_guid).await
    }
    
    async fn insert_room_mapping(&self, mapping: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        self.db.insert_room_mapping(mapping).await
    }
    
    async fn delete_room_mapping(&self, imessage_chat_guid: &str) -> Result<(), DatabaseError> {
        self.db.delete_room_mapping(imessage_chat_guid).await
    }
    
    async fn get_user_mapping(&self, imessage_user_guid: &str) -> Result<Option<UserMapping>, DatabaseError> {
        self.db.get_user_mapping(imessage_user_guid).await
    }
    
    async fn insert_user_mapping(&self, mapping: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        self.db.insert_user_mapping(mapping).await
    }
    
    async fn delete_user_mapping(&self, imessage_user_guid: &str) -> Result<(), DatabaseError> {
        self.db.delete_user_mapping(imessage_user_guid).await
    }
}

pub struct SqliteDatabase {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteDatabase {
    pub fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let url = config.url.clone();
        let manager = ConnectionManager::<SqliteConnection>::new(url);
        let pool = Pool::builder()
            .max_size(config.max_connections.unwrap_or(10))
            .min_idle(config.min_connections)
            .build(manager)
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        
        Ok(Self { pool })
    }
}

#[async_trait]
impl Database for SqliteDatabase {
    async fn migrate(&self) -> Result<(), DatabaseError> {
        info!("Running SQLite migrations");
        
        let conn = self.pool.get().map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| DatabaseError::MigrationError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn get_message_mapping(&self, _imessage_guid: &str) -> Result<Option<MessageMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn insert_message_mapping(&self, _mapping: NewMessageMapping) -> Result<MessageMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_message_mapping(&self, _imessage_guid: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn get_room_mapping(&self, _imessage_chat_guid: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn insert_room_mapping(&self, _mapping: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_room_mapping(&self, _imessage_chat_guid: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
    
    async fn get_user_mapping(&self, _imessage_user_guid: &str) -> Result<Option<UserMapping>, DatabaseError> {
        Ok(None)
    }
    
    async fn insert_user_mapping(&self, _mapping: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_user_mapping(&self, _imessage_user_guid: &str) -> Result<(), DatabaseError> {
        Ok(())
    }
}

pub struct PostgresDatabase;

impl PostgresDatabase {
    pub fn new(_config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        Err(DatabaseError::InvalidConfig("PostgreSQL not yet implemented".to_string()))
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn migrate(&self) -> Result<(), DatabaseError> {
        unimplemented!()
    }
    
    async fn get_message_mapping(&self, _imessage_guid: &str) -> Result<Option<MessageMapping>, DatabaseError> {
        unimplemented!()
    }
    
    async fn insert_message_mapping(&self, _mapping: NewMessageMapping) -> Result<MessageMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_message_mapping(&self, _imessage_guid: &str) -> Result<(), DatabaseError> {
        unimplemented!()
    }
    
    async fn get_room_mapping(&self, _imessage_chat_guid: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        unimplemented!()
    }
    
    async fn insert_room_mapping(&self, _mapping: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_room_mapping(&self, _imessage_chat_guid: &str) -> Result<(), DatabaseError> {
        unimplemented!()
    }
    
    async fn get_user_mapping(&self, _imessage_user_guid: &str) -> Result<Option<UserMapping>, DatabaseError> {
        unimplemented!()
    }
    
    async fn insert_user_mapping(&self, _mapping: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_user_mapping(&self, _imessage_user_guid: &str) -> Result<(), DatabaseError> {
        unimplemented!()
    }
}

pub struct MysqlDatabase;

impl MysqlDatabase {
    pub fn new(_config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        Err(DatabaseError::InvalidConfig("MySQL not yet implemented".to_string()))
    }
}

#[async_trait]
impl Database for MysqlDatabase {
    async fn migrate(&self) -> Result<(), DatabaseError> {
        unimplemented!()
    }
    
    async fn get_message_mapping(&self, _imessage_guid: &str) -> Result<Option<MessageMapping>, DatabaseError> {
        unimplemented!()
    }
    
    async fn insert_message_mapping(&self, _mapping: NewMessageMapping) -> Result<MessageMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_message_mapping(&self, _imessage_guid: &str) -> Result<(), DatabaseError> {
        unimplemented!()
    }
    
    async fn get_room_mapping(&self, _imessage_chat_guid: &str) -> Result<Option<RoomMapping>, DatabaseError> {
        unimplemented!()
    }
    
    async fn insert_room_mapping(&self, _mapping: NewRoomMapping) -> Result<RoomMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_room_mapping(&self, _imessage_chat_guid: &str) -> Result<(), DatabaseError> {
        unimplemented!()
    }
    
    async fn get_user_mapping(&self, _imessage_user_guid: &str) -> Result<Option<UserMapping>, DatabaseError> {
        unimplemented!()
    }
    
    async fn insert_user_mapping(&self, _mapping: NewUserMapping) -> Result<UserMapping, DatabaseError> {
        unimplemented!()
    }
    
    async fn delete_user_mapping(&self, _imessage_user_guid: &str) -> Result<(), DatabaseError> {
        unimplemented!()
    }
}
