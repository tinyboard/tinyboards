// @generated automatically by Diesel CLI.

diesel::table! {
    board (id) {
        id -> Int4,
        name -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        tag_id -> Int4,
        creator_id -> Int4,
        removed -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    board_moderator (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    board_subscriber (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    board_user_ban (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    site (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Text>,
        creator_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    tag (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    user_ (id) {
        id -> Int4,
        name -> Varchar,
        fedi_name -> Varchar,
        preferred_name -> Nullable<Varchar>,
        passhash -> Text,
        email -> Nullable<Text>,
        admin -> Bool,
        banned -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_ban (id) {
        id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::joinable!(board -> tag (tag_id));
diesel::joinable!(board -> user_ (creator_id));
diesel::joinable!(board_moderator -> board (board_id));
diesel::joinable!(board_moderator -> user_ (user_id));
diesel::joinable!(board_subscriber -> board (board_id));
diesel::joinable!(board_subscriber -> user_ (user_id));
diesel::joinable!(board_user_ban -> board (board_id));
diesel::joinable!(board_user_ban -> user_ (user_id));
diesel::joinable!(site -> user_ (creator_id));
diesel::joinable!(user_ban -> user_ (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    board,
    board_moderator,
    board_subscriber,
    board_user_ban,
    site,
    tag,
    user_,
    user_ban,
);
