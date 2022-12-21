// @generated automatically by Diesel CLI.

diesel::table! {
    admin_purge_board (id) {
        id -> Int4,
        admin_id -> Int4,
        board_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    admin_purge_comment (id) {
        id -> Int4,
        admin_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    admin_purge_post (id) {
        id -> Int4,
        admin_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    admin_purge_user (id) {
        id -> Int4,
        admin_id -> Int4,
        user_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    board_aggregates (id) {
        id -> Int4,
        board_id -> Int4,
        subscribers -> Int8,
        posts -> Int8,
        comments -> Int8,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    board_mods (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    board_subscriptions (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
        pending -> Nullable<Bool>,
    }
}

diesel::table! {
    board_user_bans (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
        expires -> Nullable<Timestamp>,
    }
}

diesel::table! {
    boards (id) {
        id -> Int4,
        name -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        tag_id -> Int4,
        creator_id -> Int4,
        is_banned -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        is_deleted -> Bool,
        is_nsfw -> Bool,
        is_hidden -> Bool,
    }
}

diesel::table! {
    comment_aggregates (id) {
        id -> Int4,
        comment_id -> Int4,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    comment_votes (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    email_verification (id) {
        id -> Int4,
        user_id -> Int4,
        email -> Text,
        verification_code -> Text,
        created -> Timestamp,
    }
}

diesel::table! {
    dms (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        body -> Text,
        is_deleted -> Bool,
        read -> Bool,
        creation_date -> Timestamp,
        edited_date -> Nullable<Timestamp>,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    mod_add_admin (id) {
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
    mod_add_board_mod (id) {
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
    password_reset_requests (id) {
        id -> Int4,
        user_id -> Int4,
        token_encrypted -> Text,
        published -> Timestamp,
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
        creation_date -> Timestamp,
        newest_comment_time -> Timestamp,
    }
}

diesel::table! {
    post_votes (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        score -> Int2,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        type_ -> Varchar,
        url -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        permalink -> Nullable<Text>,
        body -> Text,
        body_html -> Text,
        creator_id -> Int4,
        board_id -> Int4,
        is_removed -> Bool,
        is_locked -> Bool,
        creation_date -> Timestamp,
        edited_date -> Nullable<Timestamp>,
        is_deleted -> Bool,
        is_nsfw -> Bool,
        is_stickied -> Bool,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    registration_applications (id) {
        id -> Int4,
        user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published -> Timestamp,
    }
}

diesel::table! {
    secret (id) {
        id -> Int4,
        jwt_secret -> Varchar,
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
        require_application -> Bool,
        application_question -> Nullable<Text>,
        private_instance -> Bool,
        email_verification_required -> Bool,
        invite_only -> Bool,
    }
}

diesel::table! {
    site_invite (id) {
        id -> Int4,
        verification_code -> Text,
        created -> Timestamp,
    }
}

diesel::table! {
    tag (id) {
        id -> Int4,
        name -> Varchar,
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
    user_blocks (id) {
        id -> Int4,
        user_id -> Int4,
        target_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    user_board_blocks (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    user_comment_save (id) {
        id -> Int4,
        comment_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    user_mentions (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamp,
    }
}

diesel::table! {
    user_post_read (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    user_post_save (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        preferred_name -> Nullable<Varchar>,
        passhash -> Text,
        email -> Nullable<Text>,
        login_nonce -> Nullable<Int4>,
        is_admin -> Bool,
        is_banned -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        theme -> Varchar,
        default_sort_type -> Int2,
        default_listing_type -> Int2,
        avatar -> Nullable<Text>,
        email_notifications_enabled -> Bool,
        show_nsfw -> Bool,
        accepted_application -> Bool,
        is_deleted -> Bool,
        unban_date -> Nullable<Timestamp>,
        banner -> Nullable<Text>,
        bio -> Nullable<Text>,
        is_application_accepted -> Bool,
    }
}

diesel::joinable!(admin_purge_board -> boards (board_id));
diesel::joinable!(admin_purge_board -> users (admin_id));
diesel::joinable!(admin_purge_comment -> comments (comment_id));
diesel::joinable!(admin_purge_comment -> users (admin_id));
diesel::joinable!(admin_purge_post -> posts (post_id));
diesel::joinable!(admin_purge_post -> users (admin_id));
diesel::joinable!(board_aggregates -> boards (board_id));
diesel::joinable!(board_mods -> boards (board_id));
diesel::joinable!(board_mods -> users (user_id));
diesel::joinable!(board_subscriptions -> boards (board_id));
diesel::joinable!(board_subscriptions -> users (user_id));
diesel::joinable!(board_user_bans -> boards (board_id));
diesel::joinable!(board_user_bans -> users (user_id));
diesel::joinable!(boards -> tag (tag_id));
diesel::joinable!(boards -> users (creator_id));
diesel::joinable!(comment_aggregates -> comments (comment_id));
diesel::joinable!(comment_votes -> comments (comment_id));
diesel::joinable!(comment_votes -> users (user_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(comments -> users (creator_id));
diesel::joinable!(mod_add_board -> boards (board_id));
diesel::joinable!(mod_add_board_mod -> boards (board_id));
diesel::joinable!(mod_ban_from_board -> boards (board_id));
diesel::joinable!(mod_lock_post -> posts (post_id));
diesel::joinable!(mod_lock_post -> users (mod_user_id));
diesel::joinable!(mod_remove_board -> boards (board_id));
diesel::joinable!(mod_remove_board -> users (mod_user_id));
diesel::joinable!(mod_remove_comment -> comments (comment_id));
diesel::joinable!(mod_remove_comment -> users (mod_user_id));
diesel::joinable!(mod_remove_post -> posts (post_id));
diesel::joinable!(mod_remove_post -> users (mod_user_id));
diesel::joinable!(mod_sticky_post -> posts (post_id));
diesel::joinable!(mod_sticky_post -> users (mod_user_id));
diesel::joinable!(password_reset_requests -> users (user_id));
diesel::joinable!(post_aggregates -> posts (post_id));
diesel::joinable!(post_votes -> posts (post_id));
diesel::joinable!(post_votes -> users (user_id));
diesel::joinable!(posts -> boards (board_id));
diesel::joinable!(posts -> users (creator_id));
diesel::joinable!(site -> users (creator_id));
diesel::joinable!(user_aggregates -> users (user_id));
diesel::joinable!(user_ban -> users (user_id));
diesel::joinable!(user_board_blocks -> boards (board_id));
diesel::joinable!(user_board_blocks -> users (user_id));
diesel::joinable!(user_comment_save -> comments (comment_id));
diesel::joinable!(user_comment_save -> users (user_id));
diesel::joinable!(user_mentions -> comments (comment_id));
diesel::joinable!(user_mentions -> users (recipient_id));
diesel::joinable!(user_post_read -> posts (post_id));
diesel::joinable!(user_post_read -> users (user_id));
diesel::joinable!(user_post_save -> posts (post_id));
diesel::joinable!(user_post_save -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_purge_board,
    admin_purge_comment,
    admin_purge_post,
    admin_purge_user,
    board_aggregates,
    board_mods,
    board_subscriptions,
    board_user_bans,
    boards,
    comment_aggregates,
    comment_votes,
    comments,
    dms,
    email_verification,
    mod_add_admin,
    mod_add_board,
    mod_add_board_mod,
    mod_ban,
    mod_ban_from_board,
    mod_lock_post,
    mod_remove_board,
    mod_remove_comment,
    mod_remove_post,
    mod_sticky_post,
    password_reset_requests,
    post_aggregates,
    post_votes,
    posts,
    registration_applications,
    secret,
    site,
    site_invite,
    tag,
    user_aggregates,
    user_ban,
    user_blocks,
    user_board_blocks,
    user_comment_save,
    user_mentions,
    user_post_read,
    user_post_save,
    users,
);
