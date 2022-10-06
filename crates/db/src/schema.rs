// @generated automatically by Diesel CLI.

diesel::table! {
    guild (id) {
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
    guild_moderator (id) {
        id -> Int4,
        guild_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    guild_subscriber (id) {
        id -> Int4,
        guild_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    guild_user_ban (id) {
        id -> Int4,
        guild_id -> Int4,
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

diesel::joinable!(guild -> tag (tag_id));
diesel::joinable!(guild -> user_ (creator_id));
diesel::joinable!(guild_moderator -> guild (guild_id));
diesel::joinable!(guild_moderator -> user_ (user_id));
diesel::joinable!(guild_subscriber -> guild (guild_id));
diesel::joinable!(guild_subscriber -> user_ (user_id));
diesel::joinable!(guild_user_ban -> guild (guild_id));
diesel::joinable!(guild_user_ban -> user_ (user_id));
diesel::joinable!(site -> user_ (creator_id));
diesel::joinable!(user_ban -> user_ (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    guild,
    guild_moderator,
    guild_subscriber,
    guild_user_ban,
    site,
    tag,
    user_,
    user_ban,
);
