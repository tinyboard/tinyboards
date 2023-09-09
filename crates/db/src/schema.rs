// @generated automatically by Diesel CLI.

diesel::table! {
    activity (id) {
        id -> Int4,
        ap_id -> Text,
        data -> Jsonb,
        local -> Bool,
        sensitive -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

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
        person_id -> Int4,
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
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
    }
}

diesel::table! {
    board_language (id) {
        id -> Int4,
        board_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    board_mods (id) {
        id -> Int4,
        board_id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    board_person_bans (id) {
        id -> Int4,
        board_id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
        expires -> Nullable<Timestamp>,
    }
}

diesel::table! {
    board_subscriber (id) {
        id -> Int4,
        board_id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
        pending -> Bool,
    }
}

diesel::table! {
    boards (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 150]
        title -> Varchar,
        description -> Nullable<Text>,
        is_banned -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        is_deleted -> Bool,
        is_nsfw -> Bool,
        is_hidden -> Bool,
        actor_id -> Text,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        subscribers_url -> Text,
        inbox_url -> Text,
        shared_inbox_url -> Nullable<Text>,
        last_refreshed_date -> Timestamp,
        instance_id -> Int4,
        moderators_url -> Nullable<Text>,
        featured_url -> Nullable<Text>,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        posting_restricted_to_mods -> Bool,
        is_removed -> Bool,
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
    comment_reply (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    comment_report (id) {
        id -> Int4,
        creator_id -> Int4,
        comment_id -> Int4,
        original_comment_text -> Text,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    comment_saved (id) {
        id -> Int4,
        comment_id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    comment_votes (id) {
        id -> Int4,
        person_id -> Int4,
        comment_id -> Int4,
        score -> Int2,
        creation_date -> Timestamp,
        post_id -> Int4,
    }
}

diesel::table! {
    comments (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        parent_id -> Nullable<Int4>,
        body -> Text,
        body_html -> Text,
        is_removed -> Bool,
        read -> Bool,
        creation_date -> Timestamp,
        level -> Int4,
        is_deleted -> Bool,
        updated -> Nullable<Timestamp>,
        is_locked -> Bool,
        board_id -> Int4,
        local -> Bool,
        ap_id -> Nullable<Text>,
        language_id -> Int4,
    }
}

diesel::table! {
    email_verification (id) {
        id -> Int4,
        local_user_id -> Int4,
        email -> Text,
        verification_code -> Text,
        created -> Timestamp,
    }
}

diesel::table! {
    emoji (id) {
        id -> Int4,
        local_site_id -> Int4,
        #[max_length = 128]
        shortcode -> Varchar,
        image_url -> Text,
        alt_text -> Text,
        category -> Text,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    emoji_keyword (id) {
        id -> Int4,
        emoji_id -> Int4,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::table! {
    federation_allowlist (id) {
        id -> Int4,
        instance_id -> Int4,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    federation_blocklist (id) {
        id -> Int4,
        instance_id -> Int4,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        domain -> Text,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    language (id) {
        id -> Int4,
        code -> Text,
        name -> Text,
    }
}

diesel::table! {
    local_site (id) {
        id -> Int4,
        site_id -> Int4,
        site_setup -> Bool,
        invite_only -> Bool,
        enable_downvotes -> Bool,
        open_registration -> Bool,
        enable_nsfw -> Bool,
        board_creation_admin_only -> Bool,
        require_email_verification -> Bool,
        require_application -> Bool,
        application_question -> Nullable<Text>,
        private_instance -> Bool,
        default_theme -> Text,
        default_post_listing_type -> Text,
        default_avatar -> Nullable<Text>,
        legal_information -> Nullable<Text>,
        hide_modlog_mod_names -> Bool,
        application_email_admins -> Bool,
        actor_name_max_length -> Int4,
        federation_enabled -> Bool,
        federation_debug -> Bool,
        federation_strict_allowlist -> Bool,
        federation_http_fetch_retry_limit -> Int4,
        federation_worker_count -> Int4,
        captcha_enabled -> Bool,
        #[max_length = 255]
        captcha_difficulty -> Varchar,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        reports_email_admins -> Bool,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 25]
        primary_color -> Nullable<Varchar>,
        #[max_length = 25]
        secondary_color -> Nullable<Varchar>,
        #[max_length = 25]
        hover_color -> Nullable<Varchar>,
        #[max_length = 255]
        description -> Nullable<Varchar>,
        #[max_length = 255]
        icon -> Nullable<Varchar>,
        #[max_length = 255]
        welcome_message -> Nullable<Varchar>,
    }
}

diesel::table! {
    local_site_rate_limit (id) {
        id -> Int4,
        local_site_id -> Int4,
        message -> Int4,
        message_per_second -> Int4,
        post -> Int4,
        post_per_second -> Int4,
        register -> Int4,
        register_per_second -> Int4,
        image -> Int4,
        image_per_second -> Int4,
        comment -> Int4,
        comment_per_second -> Int4,
        search -> Int4,
        search_per_second -> Int4,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    local_user (id) {
        id -> Int4,
        name -> Text,
        person_id -> Int4,
        passhash -> Text,
        email -> Nullable<Text>,
        is_admin -> Bool,
        is_banned -> Bool,
        is_deleted -> Bool,
        unban_date -> Nullable<Timestamp>,
        show_nsfw -> Bool,
        show_bots -> Bool,
        theme -> Text,
        default_sort_type -> Int2,
        default_listing_type -> Int2,
        interface_language -> Text,
        email_notifications_enabled -> Bool,
        accepted_application -> Bool,
        is_application_accepted -> Bool,
        email_verified -> Bool,
        updated -> Nullable<Timestamp>,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    local_user_language (id) {
        id -> Int4,
        local_user_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    mod_add_admin (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_add_board (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        board_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_add_board_mod (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        board_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_ban (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_ban_from_board (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        board_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_feature_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        featured -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_hide_board (id) {
        id -> Int4,
        board_id -> Int4,
        mod_person_id -> Int4,
        when_ -> Timestamp,
        reason -> Nullable<Text>,
        hidden -> Bool,
    }
}

diesel::table! {
    mod_lock_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        locked -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_board (id) {
        id -> Int4,
        mod_person_id -> Int4,
        board_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_comment (id) {
        id -> Int4,
        mod_person_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    password_resets (id) {
        id -> Int4,
        reset_token -> Text,
        creation_date -> Timestamp,
        local_user_id -> Int4,
    }
}

diesel::table! {
    person (id) {
        id -> Int4,
        #[max_length = 30]
        name -> Varchar,
        #[max_length = 30]
        display_name -> Nullable<Varchar>,
        is_banned -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        avatar -> Nullable<Text>,
        is_deleted -> Bool,
        unban_date -> Nullable<Timestamp>,
        banner -> Nullable<Text>,
        bio -> Nullable<Text>,
        signature -> Nullable<Text>,
        actor_id -> Text,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        inbox_url -> Text,
        shared_inbox_url -> Nullable<Text>,
        bot_account -> Bool,
        last_refreshed_date -> Timestamp,
        instance_id -> Int4,
        is_admin -> Bool,
        #[max_length = 256]
        instance -> Nullable<Varchar>,
    }
}

diesel::table! {
    person_aggregates (id) {
        id -> Int4,
        person_id -> Int4,
        post_count -> Int8,
        post_score -> Int8,
        comment_count -> Int8,
        comment_score -> Int8,
        rep -> Int8,
    }
}

diesel::table! {
    person_ban (id) {
        id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    person_blocks (id) {
        id -> Int4,
        person_id -> Int4,
        target_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    person_board_blocks (id) {
        id -> Int4,
        person_id -> Int4,
        board_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    person_mentions (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    person_subscriber (id) {
        id -> Int4,
        person_id -> Int4,
        subscriber_id -> Int4,
        creation_date -> Timestamp,
        pending -> Bool,
    }
}

diesel::table! {
    pm_notif (id) {
        id -> Int4,
        recipient_id -> Int4,
        pm_id -> Int4,
        read -> Bool,
        creation_date -> Timestamp,
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
        is_stickied -> Bool,
        creation_date -> Timestamp,
        newest_comment_time -> Timestamp,
    }
}

diesel::table! {
    post_read (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    post_report (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        original_post_title -> Text,
        original_post_url -> Nullable<Text>,
        original_post_body -> Nullable<Text>,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    post_saved (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    post_votes (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        score -> Int2,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        #[max_length = 200]
        title -> Varchar,
        #[max_length = 10]
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
        is_deleted -> Bool,
        is_nsfw -> Bool,
        updated -> Nullable<Timestamp>,
        image -> Nullable<Text>,
        language_id -> Int4,
        ap_id -> Nullable<Text>,
        local -> Bool,
        featured_board -> Bool,
        featured_local -> Bool,
    }
}

diesel::table! {
    private_message (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_user_id -> Nullable<Int4>,
        recipient_board_id -> Nullable<Int4>,
        body -> Text,
        body_html -> Text,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    registration_applications (id) {
        id -> Int4,
        person_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        creation_date -> Timestamp,
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
        #[max_length = 20]
        name -> Varchar,
        sidebar -> Nullable<Text>,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        actor_id -> Text,
        instance_id -> Int4,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        description -> Nullable<Text>,
        last_refreshed_date -> Timestamp,
        inbox_url -> Text,
        private_key -> Nullable<Text>,
        public_key -> Text,
    }
}

diesel::table! {
    site_aggregates (id) {
        id -> Int4,
        site_id -> Int4,
        users -> Int8,
        posts -> Int8,
        comments -> Int8,
        boards -> Int8,
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
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
    site_language (id) {
        id -> Int4,
        site_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    stray_images (id) {
        id -> Int4,
        img_url -> Text,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    uploads (id) {
        id -> Int4,
        person_id -> Int4,
        original_name -> Text,
        file_name -> Text,
        file_path -> Text,
        upload_url -> Text,
        creation_date -> Timestamp,
        size -> Int8,
    }
}

diesel::joinable!(admin_purge_board -> boards (board_id));
diesel::joinable!(admin_purge_board -> person (admin_id));
diesel::joinable!(admin_purge_comment -> comments (comment_id));
diesel::joinable!(admin_purge_comment -> person (admin_id));
diesel::joinable!(admin_purge_post -> person (admin_id));
diesel::joinable!(admin_purge_post -> posts (post_id));
diesel::joinable!(board_aggregates -> boards (board_id));
diesel::joinable!(board_language -> boards (board_id));
diesel::joinable!(board_language -> language (language_id));
diesel::joinable!(board_mods -> boards (board_id));
diesel::joinable!(board_mods -> person (person_id));
diesel::joinable!(board_person_bans -> boards (board_id));
diesel::joinable!(board_person_bans -> person (person_id));
diesel::joinable!(board_subscriber -> boards (board_id));
diesel::joinable!(board_subscriber -> person (person_id));
diesel::joinable!(boards -> instance (instance_id));
diesel::joinable!(comment_aggregates -> comments (comment_id));
diesel::joinable!(comment_reply -> comments (comment_id));
diesel::joinable!(comment_reply -> person (recipient_id));
diesel::joinable!(comment_report -> comments (comment_id));
diesel::joinable!(comment_saved -> comments (comment_id));
diesel::joinable!(comment_saved -> person (person_id));
diesel::joinable!(comment_votes -> comments (comment_id));
diesel::joinable!(comment_votes -> person (person_id));
diesel::joinable!(comment_votes -> posts (post_id));
diesel::joinable!(comments -> language (language_id));
diesel::joinable!(comments -> person (creator_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(email_verification -> person (local_user_id));
diesel::joinable!(emoji -> local_site (local_site_id));
diesel::joinable!(emoji_keyword -> emoji (emoji_id));
diesel::joinable!(federation_allowlist -> instance (instance_id));
diesel::joinable!(federation_blocklist -> instance (instance_id));
diesel::joinable!(local_site -> site (site_id));
diesel::joinable!(local_site_rate_limit -> local_site (local_site_id));
diesel::joinable!(local_user -> person (person_id));
diesel::joinable!(local_user_language -> language (language_id));
diesel::joinable!(local_user_language -> local_user (local_user_id));
diesel::joinable!(mod_add_board -> boards (board_id));
diesel::joinable!(mod_add_board_mod -> boards (board_id));
diesel::joinable!(mod_ban_from_board -> boards (board_id));
diesel::joinable!(mod_feature_post -> person (mod_person_id));
diesel::joinable!(mod_feature_post -> posts (post_id));
diesel::joinable!(mod_hide_board -> boards (board_id));
diesel::joinable!(mod_hide_board -> person (mod_person_id));
diesel::joinable!(mod_lock_post -> person (mod_person_id));
diesel::joinable!(mod_lock_post -> posts (post_id));
diesel::joinable!(mod_remove_board -> boards (board_id));
diesel::joinable!(mod_remove_board -> person (mod_person_id));
diesel::joinable!(mod_remove_comment -> comments (comment_id));
diesel::joinable!(mod_remove_comment -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> posts (post_id));
diesel::joinable!(password_resets -> local_user (local_user_id));
diesel::joinable!(person -> instance (instance_id));
diesel::joinable!(person_aggregates -> person (person_id));
diesel::joinable!(person_ban -> person (person_id));
diesel::joinable!(person_board_blocks -> boards (board_id));
diesel::joinable!(person_board_blocks -> person (person_id));
diesel::joinable!(person_mentions -> comments (comment_id));
diesel::joinable!(person_mentions -> person (recipient_id));
diesel::joinable!(pm_notif -> person (recipient_id));
diesel::joinable!(pm_notif -> private_message (pm_id));
diesel::joinable!(post_aggregates -> posts (post_id));
diesel::joinable!(post_read -> person (person_id));
diesel::joinable!(post_read -> posts (post_id));
diesel::joinable!(post_report -> posts (post_id));
diesel::joinable!(post_saved -> person (person_id));
diesel::joinable!(post_saved -> posts (post_id));
diesel::joinable!(post_votes -> person (person_id));
diesel::joinable!(post_votes -> posts (post_id));
diesel::joinable!(posts -> boards (board_id));
diesel::joinable!(posts -> language (language_id));
diesel::joinable!(posts -> person (creator_id));
diesel::joinable!(site -> instance (instance_id));
diesel::joinable!(site_aggregates -> site (site_id));
diesel::joinable!(site_language -> language (language_id));
diesel::joinable!(site_language -> site (site_id));
diesel::joinable!(uploads -> person (person_id));

diesel::allow_tables_to_appear_in_same_query!(
    activity,
    admin_purge_board,
    admin_purge_comment,
    admin_purge_post,
    admin_purge_user,
    board_aggregates,
    board_language,
    board_mods,
    board_person_bans,
    board_subscriber,
    boards,
    comment_aggregates,
    comment_reply,
    comment_report,
    comment_saved,
    comment_votes,
    comments,
    email_verification,
    emoji,
    emoji_keyword,
    federation_allowlist,
    federation_blocklist,
    instance,
    language,
    local_site,
    local_site_rate_limit,
    local_user,
    local_user_language,
    mod_add_admin,
    mod_add_board,
    mod_add_board_mod,
    mod_ban,
    mod_ban_from_board,
    mod_feature_post,
    mod_hide_board,
    mod_lock_post,
    mod_remove_board,
    mod_remove_comment,
    mod_remove_post,
    password_resets,
    person,
    person_aggregates,
    person_ban,
    person_blocks,
    person_board_blocks,
    person_mentions,
    person_subscriber,
    pm_notif,
    post_aggregates,
    post_read,
    post_report,
    post_saved,
    post_votes,
    posts,
    private_message,
    registration_applications,
    secret,
    site,
    site_aggregates,
    site_invite,
    site_language,
    stray_images,
    uploads,
);
