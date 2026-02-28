use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crate::db::schema::message_mappings)]
pub struct MessageMapping {
    pub id: i32,
    pub imessage_guid: String,
    pub matrix_event_id: String,
    pub matrix_room_id: String,
    pub channel_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crate::db::schema::room_mappings)]
pub struct RoomMapping {
    pub id: i32,
    pub imessage_chat_guid: String,
    pub matrix_room_id: String,
    pub channel_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crate::db::schema::user_mappings)]
pub struct UserMapping {
    pub id: i32,
    pub imessage_user_guid: String,
    pub matrix_user_id: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMessageMapping {
    pub imessage_guid: String,
    pub matrix_event_id: String,
    pub matrix_room_id: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRoomMapping {
    pub imessage_chat_guid: String,
    pub matrix_room_id: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUserMapping {
    pub imessage_user_guid: String,
    pub matrix_user_id: String,
}
