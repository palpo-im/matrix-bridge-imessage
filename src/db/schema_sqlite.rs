use diesel::{allow_tables_to_appear_in_same_query, table};

table! {
    message_mappings (id) {
        id -> Integer,
        imessage_guid -> Text,
        matrix_event_id -> Text,
        matrix_room_id -> Text,
        channel_id -> Text,
        created_at -> Timestamp,
    }
}

table! {
    room_mappings (id) {
        id -> Integer,
        imessage_chat_guid -> Text,
        matrix_room_id -> Text,
        channel_id -> Text,
        created_at -> Timestamp,
    }
}

table! {
    user_mappings (id) {
        id -> Integer,
        imessage_user_guid -> Text,
        matrix_user_id -> Text,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(message_mappings, room_mappings, user_mappings,);
