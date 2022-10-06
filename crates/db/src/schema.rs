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
        deleted -> Bool,
        nsfw -> Bool,
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
    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        parent_id -> Nullable<Int4>,
        body -> Text,
        removed -> Bool,
        read -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
    }
}

diesel::table! {
    comment_like (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        post_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

diesel::table! {
    comment_saved (id) {
        id -> Int4,
        comment_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    post (id) {
        id -> Int4,
        name -> Varchar,
        url -> Nullable<Text>,
        body -> Text,
        creator_id -> Int4,
        board_id -> Int4,
        removed -> Bool,
        locked -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
        nsfw -> Bool,
    }
}

diesel::table! {
    post_like (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        score -> Int2,
    }
}

diesel::table! {
    post_read (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    post_saved (id) {
        id -> Int4,
        post_id -> Int4,
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
        show_nsfw -> Bool,
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
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment -> user_ (creator_id));
diesel::joinable!(comment_like -> comment (comment_id));
diesel::joinable!(comment_like -> post (post_id));
diesel::joinable!(comment_like -> user_ (user_id));
diesel::joinable!(comment_saved -> comment (comment_id));
diesel::joinable!(comment_saved -> user_ (user_id));
diesel::joinable!(post -> board (board_id));
diesel::joinable!(post -> user_ (creator_id));
diesel::joinable!(post_like -> post (post_id));
diesel::joinable!(post_like -> user_ (user_id));
diesel::joinable!(post_read -> post (post_id));
diesel::joinable!(post_read -> user_ (user_id));
diesel::joinable!(post_saved -> post (post_id));
diesel::joinable!(post_saved -> user_ (user_id));
diesel::joinable!(site -> user_ (creator_id));
diesel::joinable!(user_ban -> user_ (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    board,
    board_moderator,
    board_subscriber,
    board_user_ban,
    comment,
    comment_like,
    comment_saved,
    post,
    post_like,
    post_read,
    post_saved,
    site,
    tag,
    user_,
    user_ban,
);
