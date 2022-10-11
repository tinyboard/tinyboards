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
    board_aggregates (id) {
        id -> Int4,
        board_id -> Int4,
        subscribers -> Int8,
        posts -> Int8,
        comments -> Int8,
        published -> Timestamp,
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
    comment_aggregates (id) {
        id -> Int4,
        comment_id -> Int4,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        published -> Timestamp,
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
    mod_add (id) {
        id -> Int4,
        mod_user_id -> Int4,
        other_user_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_add_board (id) {
        id -> Int4,
        mod_user_id -> Int4,
        other_user_id -> Int4,
        board_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_ban (id) {
        id -> Int4,
        mod_user_id -> Int4,
        other_user_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_ban_from_board (id) {
        id -> Int4,
        mod_user_id -> Int4,
        other_user_id -> Int4,
        board_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_lock_post (id) {
        id -> Int4,
        mod_user_id -> Int4,
        post_id -> Int4,
        locked -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_board (id) {
        id -> Int4,
        mod_user_id -> Int4,
        board_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_comment (id) {
        id -> Int4,
        mod_user_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_post (id) {
        id -> Int4,
        mod_user_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_sticky_post (id) {
        id -> Int4,
        mod_user_id -> Int4,
        post_id -> Int4,
        stickied -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    password_reset_request (id) {
        id -> Int4,
        user_id -> Int4,
        token_encrypted -> Text,
        published -> Timestamp,
    }
}

diesel::table! {
    post (id) {
        id -> Int4,
        title -> Varchar,
        type_ -> Varchar,
        url -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        permalink -> Nullable<Text>,
        body -> Text,
        creator_id -> Int4,
        board_id -> Int4,
        removed -> Bool,
        locked -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
        nsfw -> Bool,
        stickied -> Bool,
    }
}

diesel::table! {
    post_aggregates (id) {
        id -> Int4,
        post_id -> Int4,
        comments -> Int8,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        stickied -> Bool,
        published -> Timestamp,
        newest_comment_time -> Timestamp,
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
    private_message (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        body -> Text,
        deleted -> Bool,
        read -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
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
        enable_downvotes -> Bool,
        open_registration -> Bool,
        enable_nsfw -> Bool,
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
        theme -> Varchar,
        default_sort_type -> Int2,
        default_listing_type -> Int2,
        avatar -> Nullable<Text>,
        email_notifications_enabled -> Bool,
        show_nsfw -> Bool,
    }
}

diesel::table! {
    user_aggregates (id) {
        id -> Int4,
        user_id -> Int4,
        post_count -> Int8,
        post_score -> Int8,
        comment_count -> Int8,
        comment_score -> Int8,
    }
}

diesel::table! {
    user_ban (id) {
        id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    user_mention (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamp,
    }
}

diesel::joinable!(board -> tag (tag_id));
diesel::joinable!(board -> user_ (creator_id));
diesel::joinable!(board_aggregates -> board (board_id));
diesel::joinable!(board_moderator -> board (board_id));
diesel::joinable!(board_moderator -> user_ (user_id));
diesel::joinable!(board_subscriber -> board (board_id));
diesel::joinable!(board_subscriber -> user_ (user_id));
diesel::joinable!(board_user_ban -> board (board_id));
diesel::joinable!(board_user_ban -> user_ (user_id));
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment -> user_ (creator_id));
diesel::joinable!(comment_aggregates -> comment (comment_id));
diesel::joinable!(comment_like -> comment (comment_id));
diesel::joinable!(comment_like -> post (post_id));
diesel::joinable!(comment_like -> user_ (user_id));
diesel::joinable!(comment_saved -> comment (comment_id));
diesel::joinable!(comment_saved -> user_ (user_id));
diesel::joinable!(mod_add_board -> board (board_id));
diesel::joinable!(mod_ban_from_board -> board (board_id));
diesel::joinable!(mod_lock_post -> post (post_id));
diesel::joinable!(mod_lock_post -> user_ (mod_user_id));
diesel::joinable!(mod_remove_board -> board (board_id));
diesel::joinable!(mod_remove_board -> user_ (mod_user_id));
diesel::joinable!(mod_remove_comment -> comment (comment_id));
diesel::joinable!(mod_remove_comment -> user_ (mod_user_id));
diesel::joinable!(mod_remove_post -> post (post_id));
diesel::joinable!(mod_remove_post -> user_ (mod_user_id));
diesel::joinable!(mod_sticky_post -> post (post_id));
diesel::joinable!(mod_sticky_post -> user_ (mod_user_id));
diesel::joinable!(password_reset_request -> user_ (user_id));
diesel::joinable!(post -> board (board_id));
diesel::joinable!(post -> user_ (creator_id));
diesel::joinable!(post_aggregates -> post (post_id));
diesel::joinable!(post_like -> post (post_id));
diesel::joinable!(post_like -> user_ (user_id));
diesel::joinable!(post_read -> post (post_id));
diesel::joinable!(post_read -> user_ (user_id));
diesel::joinable!(post_saved -> post (post_id));
diesel::joinable!(post_saved -> user_ (user_id));
diesel::joinable!(site -> user_ (creator_id));
diesel::joinable!(user_aggregates -> user_ (user_id));
diesel::joinable!(user_ban -> user_ (user_id));
diesel::joinable!(user_mention -> comment (comment_id));
diesel::joinable!(user_mention -> user_ (recipient_id));

diesel::allow_tables_to_appear_in_same_query!(
    board,
    board_aggregates,
    board_moderator,
    board_subscriber,
    board_user_ban,
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    mod_add,
    mod_add_board,
    mod_ban,
    mod_ban_from_board,
    mod_lock_post,
    mod_remove_board,
    mod_remove_comment,
    mod_remove_post,
    mod_sticky_post,
    password_reset_request,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
    private_message,
    site,
    tag,
    user_,
    user_aggregates,
    user_ban,
    user_mention,
);
