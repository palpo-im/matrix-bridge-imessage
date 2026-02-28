table! {
    message_mappings (id) {
        id -> Int4,
        imessage_guid -> Varchar,
        matrix_event_id -> Varchar,
        matrix_room_id -> Varchar,
        channel_id -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    room_mappings (id) {
        id -> Int4,
        imessage_chat_guid -> Varchar,
        matrix_room_id -> Varchar,
        channel_id -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    user_mappings (id) {
        id -> Int4,
        imessage_user_guid -> Varchar,
        matrix_user_id -> Varchar,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(message_mappings, room_mappings, user_mappings,);
