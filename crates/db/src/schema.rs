// @generated automatically by Diesel CLI.

diesel::table! {
    admin_ban_board (id) {
        id -> Int4,
        admin_id -> Int4,
        board_id -> Int4,
        internal_notes -> Nullable<Text>,
        public_ban_reason -> Nullable<Text>,
        #[max_length = 10]
        action -> Varchar,
        when_ -> Timestamp,
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
        user_id -> Int4,
        creation_date -> Timestamp,
        permissions -> Int4,
        rank -> Int4,
        invite_accepted -> Bool,
        invite_accepted_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    board_reaction_settings (id) {
        id -> Int4,
        board_id -> Int4,
        emoji_weights -> Jsonb,
        reactions_enabled -> Bool,
    }
}

diesel::table! {
    board_subscriber (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
        pending -> Bool,
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
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 150]
        title -> Varchar,
        description -> Nullable<Text>,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        is_deleted -> Bool,
        is_nsfw -> Bool,
        is_hidden -> Bool,
        last_refreshed_date -> Timestamp,
        instance_id -> Int4,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        posting_restricted_to_mods -> Bool,
        is_removed -> Bool,
        #[max_length = 512]
        ban_reason -> Nullable<Varchar>,
        #[max_length = 25]
        primary_color -> Varchar,
        #[max_length = 25]
        secondary_color -> Varchar,
        #[max_length = 25]
        hover_color -> Varchar,
        #[max_length = 10000]
        sidebar -> Nullable<Varchar>,
        sidebar_html -> Nullable<Text>,
        is_banned -> Bool,
        public_ban_reason -> Nullable<Text>,
        banned_by -> Nullable<Int4>,
        banned_at -> Nullable<Timestamp>,
        exclude_from_all -> Bool,
        moderators_url -> Nullable<Text>,
        featured_url -> Nullable<Text>,
        section_config -> Int4,
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
        child_count -> Int4,
        hot_rank -> Int4,
        controversy_rank -> Float8,
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
    comment_reports (id) {
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
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    comment_votes (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        post_id -> Int4,
        score -> Int4,
        creation_date -> Timestamp,
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
        language_id -> Nullable<Int4>,
        is_pinned -> Nullable<Bool>,
        #[max_length = 20]
        approval_status -> Varchar,
        approved_by -> Nullable<Int4>,
        approved_at -> Nullable<Timestamp>,
        creator_vote -> Int4,
        quoted_comment_id -> Nullable<Int4>,
    }
}

diesel::table! {
    content_uploads (id) {
        id -> Int4,
        upload_id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        created_at -> Timestamp,
        position -> Nullable<Int4>,
    }
}

diesel::table! {
    email_verification (id) {
        id -> Int4,
        user_id -> Int4,
        email -> Text,
        verification_code -> Text,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    emoji (id) {
        id -> Int4,
        #[max_length = 128]
        shortcode -> Varchar,
        image_url -> Text,
        alt_text -> Text,
        category -> Text,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        board_id -> Nullable<Int4>,
        created_by_user_id -> Int4,
        is_active -> Bool,
        usage_count -> Int4,
        #[max_length = 10]
        emoji_scope -> Varchar,
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
    language (id) {
        id -> Int4,
        #[max_length = 3]
        code -> Varchar,
        name -> Text,
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
    messages (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        #[max_length = 200]
        subject -> Varchar,
        body -> Text,
        body_html -> Text,
        read -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
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
    mod_feature_post (id) {
        id -> Int4,
        mod_user_id -> Int4,
        post_id -> Int4,
        featured -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_hide_board (id) {
        id -> Int4,
        board_id -> Int4,
        mod_user_id -> Int4,
        when_ -> Timestamp,
        reason -> Nullable<Text>,
        hidden -> Bool,
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
    moderation_log (id) {
        id -> Int4,
        moderator_id -> Int4,
        #[max_length = 50]
        action_type -> Varchar,
        #[max_length = 20]
        target_type -> Varchar,
        target_id -> Int4,
        board_id -> Nullable<Int4>,
        reason -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        created_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    notification_settings (id) {
        id -> Int4,
        user_id -> Int4,
        email_enabled -> Bool,
        comment_replies_enabled -> Bool,
        post_replies_enabled -> Bool,
        mentions_enabled -> Bool,
        post_votes_enabled -> Bool,
        comment_votes_enabled -> Bool,
        private_messages_enabled -> Bool,
        board_invites_enabled -> Bool,
        moderator_actions_enabled -> Bool,
        system_notifications_enabled -> Bool,
        created -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    notifications (id) {
        id -> Int4,
        kind -> Text,
        recipient_user_id -> Int4,
        comment_id -> Nullable<Int4>,
        post_id -> Nullable<Int4>,
        message_id -> Nullable<Int4>,
        created -> Timestamp,
        is_read -> Bool,
    }
}

diesel::table! {
    password_resets (id) {
        id -> Int4,
        reset_token -> Text,
        creation_date -> Timestamp,
        user_id -> Int4,
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
        creation_date -> Timestamp,
        newest_comment_time_necro -> Nullable<Timestamp>,
        newest_comment_time -> Timestamp,
        featured_board -> Bool,
        featured_local -> Bool,
        hot_rank -> Int4,
        hot_rank_active -> Int4,
        board_id -> Int4,
        creator_id -> Int4,
        controversy_rank -> Float8,
    }
}

diesel::table! {
    post_hidden (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    post_read (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
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
    post_reports (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        #[max_length = 200]
        original_post_title -> Varchar,
        original_post_url -> Nullable<Text>,
        original_post_body -> Text,
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
        user_id -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    post_votes (id) {
        id -> Int4,
        user_id -> Int4,
        post_id -> Int4,
        score -> Int4,
        creation_date -> Timestamp,
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
        language_id -> Nullable<Int4>,
        featured_board -> Bool,
        featured_local -> Bool,
        alt_text -> Nullable<Text>,
        embed_title -> Nullable<Text>,
        embed_description -> Nullable<Text>,
        embed_video_url -> Nullable<Text>,
        source_url -> Nullable<Text>,
        last_crawl_date -> Nullable<Timestamp>,
        #[max_length = 255]
        title_chunk -> Varchar,
        #[max_length = 20]
        approval_status -> Varchar,
        approved_by -> Nullable<Int4>,
        approved_at -> Nullable<Timestamp>,
        creator_vote -> Int4,
        #[max_length = 10]
        post_type -> Varchar,
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
        is_sender_hidden -> Bool,
        title -> Text,
    }
}

diesel::table! {
    reaction_aggregates (id) {
        id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        #[max_length = 100]
        emoji -> Varchar,
        count -> Int4,
    }
}

diesel::table! {
    reactions (id) {
        id -> Int4,
        user_id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        #[max_length = 100]
        emoji -> Varchar,
        score -> Int4,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    registration_applications (id) {
        id -> Int4,
        user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        creation_date -> Timestamp,
    }
}

diesel::table! {
    relations (id) {
        id -> Int4,
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
        boards_enabled -> Bool,
        #[max_length = 20]
        board_creation_mode -> Varchar,
        trusted_user_min_reputation -> Int4,
        trusted_user_min_account_age_days -> Int4,
        trusted_user_manual_approval -> Bool,
        trusted_user_min_posts -> Int4,
        allowed_post_types -> Nullable<Text>,
        enable_nsfw_tagging -> Nullable<Bool>,
        word_filter_enabled -> Nullable<Bool>,
        filtered_words -> Nullable<Text>,
        word_filter_applies_to_posts -> Nullable<Bool>,
        word_filter_applies_to_comments -> Nullable<Bool>,
        word_filter_applies_to_usernames -> Nullable<Bool>,
        link_filter_enabled -> Nullable<Bool>,
        banned_domains -> Nullable<Text>,
        approved_image_hosts -> Nullable<Text>,
        image_embed_hosts_only -> Nullable<Bool>,
        registration_mode -> Varchar,
        emoji_enabled -> Bool,
        max_emojis_per_post -> Nullable<Int4>,
        max_emojis_per_comment -> Nullable<Int4>,
        emoji_max_file_size_mb -> Int4,
        board_emojis_enabled -> Bool,
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
        upvotes -> Int8,
        downvotes -> Int8,
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
        user_id -> Int4,
        original_name -> Text,
        file_name -> Text,
        file_path -> Text,
        upload_url -> Text,
        creation_date -> Timestamp,
        size -> Int8,
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
        creation_date -> Timestamp,
        banned_by -> Nullable<Int4>,
        reason -> Nullable<Text>,
        expires_at -> Nullable<Timestamp>,
        banned_at -> Timestamp,
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
    user_language (id) {
        id -> Int4,
        user_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    user_subscriber (id) {
        id -> Int4,
        user_id -> Int4,
        subscriber_id -> Int4,
        creation_date -> Timestamp,
        pending -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 30]
        name -> Varchar,
        #[max_length = 30]
        display_name -> Nullable<Varchar>,
        email -> Nullable<Text>,
        passhash -> Text,
        email_verified -> Bool,
        is_banned -> Bool,
        is_deleted -> Bool,
        is_admin -> Bool,
        admin_level -> Int4,
        unban_date -> Nullable<Timestamp>,
        bio -> Nullable<Text>,
        bio_html -> Nullable<Text>,
        signature -> Nullable<Text>,
        avatar -> Nullable<Text>,
        banner -> Nullable<Text>,
        profile_background -> Nullable<Text>,
        avatar_frame -> Nullable<Text>,
        profile_music -> Nullable<Text>,
        profile_music_youtube -> Nullable<Text>,
        show_nsfw -> Bool,
        show_bots -> Bool,
        theme -> Text,
        default_sort_type -> Int2,
        default_listing_type -> Int2,
        interface_language -> Text,
        email_notifications_enabled -> Bool,
        bot_account -> Bool,
        board_creation_approved -> Bool,
        accepted_application -> Bool,
        is_application_accepted -> Bool,
        creation_date -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::joinable!(admin_purge_comment -> comments (comment_id));
diesel::joinable!(admin_purge_comment -> users (admin_id));
diesel::joinable!(admin_purge_post -> posts (post_id));
diesel::joinable!(admin_purge_post -> users (admin_id));
diesel::joinable!(board_aggregates -> boards (board_id));
diesel::joinable!(board_language -> boards (board_id));
diesel::joinable!(board_language -> language (language_id));
diesel::joinable!(board_mods -> boards (board_id));
diesel::joinable!(board_mods -> users (user_id));
diesel::joinable!(board_reaction_settings -> boards (board_id));
diesel::joinable!(board_subscriber -> boards (board_id));
diesel::joinable!(board_subscriber -> users (user_id));
diesel::joinable!(board_user_bans -> boards (board_id));
diesel::joinable!(board_user_bans -> users (user_id));
diesel::joinable!(comment_aggregates -> comments (comment_id));
diesel::joinable!(comment_reports -> comments (comment_id));
diesel::joinable!(comment_saved -> comments (comment_id));
diesel::joinable!(comment_saved -> users (user_id));
diesel::joinable!(comment_votes -> comments (comment_id));
diesel::joinable!(comment_votes -> posts (post_id));
diesel::joinable!(comment_votes -> users (user_id));
diesel::joinable!(comments -> language (language_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(content_uploads -> comments (comment_id));
diesel::joinable!(content_uploads -> posts (post_id));
diesel::joinable!(content_uploads -> uploads (upload_id));
diesel::joinable!(email_verification -> users (user_id));
diesel::joinable!(emoji_keyword -> emoji (emoji_id));
diesel::joinable!(moderation_log -> boards (board_id));
diesel::joinable!(moderation_log -> users (moderator_id));
diesel::joinable!(notification_settings -> users (user_id));
diesel::joinable!(notifications -> private_message (message_id));
diesel::joinable!(password_resets -> users (user_id));
diesel::joinable!(pm_notif -> private_message (pm_id));
diesel::joinable!(post_aggregates -> posts (post_id));
diesel::joinable!(post_hidden -> posts (post_id));
diesel::joinable!(post_hidden -> users (user_id));
diesel::joinable!(post_read -> posts (post_id));
diesel::joinable!(post_read -> users (user_id));
diesel::joinable!(post_reports -> posts (post_id));
diesel::joinable!(post_saved -> posts (post_id));
diesel::joinable!(post_saved -> users (user_id));
diesel::joinable!(post_votes -> posts (post_id));
diesel::joinable!(post_votes -> users (user_id));
diesel::joinable!(posts -> boards (board_id));
diesel::joinable!(posts -> language (language_id));
diesel::joinable!(reaction_aggregates -> comments (comment_id));
diesel::joinable!(reaction_aggregates -> posts (post_id));
diesel::joinable!(reactions -> comments (comment_id));
diesel::joinable!(reactions -> posts (post_id));
diesel::joinable!(reactions -> users (user_id));
diesel::joinable!(site_aggregates -> site (site_id));
diesel::joinable!(site_language -> language (language_id));
diesel::joinable!(site_language -> site (site_id));
diesel::joinable!(uploads -> users (user_id));
diesel::joinable!(user_aggregates -> users (user_id));
diesel::joinable!(user_ban -> users (banned_by));
diesel::joinable!(user_board_blocks -> boards (board_id));
diesel::joinable!(user_board_blocks -> users (user_id));
diesel::joinable!(user_language -> language (language_id));
diesel::joinable!(user_language -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    admin_ban_board,
    admin_purge_board,
    admin_purge_comment,
    admin_purge_post,
    admin_purge_user,
    board_aggregates,
    board_language,
    board_mods,
    board_reaction_settings,
    board_subscriber,
    board_user_bans,
    boards,
    comment_aggregates,
    comment_report,
    comment_reports,
    comment_saved,
    comment_votes,
    comments,
    content_uploads,
    email_verification,
    emoji,
    emoji_keyword,
    language,
    local_site_rate_limit,
    messages,
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
    moderation_log,
    notification_settings,
    notifications,
    password_resets,
    pm_notif,
    post_aggregates,
    post_hidden,
    post_read,
    post_report,
    post_reports,
    post_saved,
    post_votes,
    posts,
    private_message,
    reaction_aggregates,
    reactions,
    registration_applications,
    relations,
    secret,
    site,
    site_aggregates,
    site_invite,
    site_language,
    stray_images,
    uploads,
    user_aggregates,
    user_ban,
    user_blocks,
    user_board_blocks,
    user_language,
    user_subscriber,
    users,
);
