// Diesel schema definitions for the tinyboards database.
// Matches the new schema created by the 2026-03-03 migration set.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "registration_mode"))]
    pub struct RegistrationMode;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "post_type"))]
    pub struct PostType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "sort_type"))]
    pub struct SortType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "listing_type"))]
    pub struct ListingType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "approval_status"))]
    pub struct ApprovalStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "editor_mode"))]
    pub struct EditorMode;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_kind"))]
    pub struct NotificationKind;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "moderation_action"))]
    pub struct ModerationAction;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "wiki_permission"))]
    pub struct WikiPermission;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "flair_type"))]
    pub struct FlairType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "filter_mode"))]
    pub struct FilterMode;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "emoji_scope"))]
    pub struct EmojiScope;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "report_status"))]
    pub struct ReportStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    site (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 255]
        description -> Nullable<Varchar>,
        #[max_length = 255]
        icon -> Nullable<Varchar>,
        homepage_banner -> Nullable<Text>,
        #[max_length = 25]
        primary_color -> Varchar,
        #[max_length = 25]
        secondary_color -> Varchar,
        #[max_length = 25]
        hover_color -> Varchar,
        #[max_length = 255]
        welcome_message -> Nullable<Varchar>,
        legal_information -> Nullable<Text>,
        default_theme -> Text,
        default_post_listing_type -> ListingType,
        default_avatar -> Nullable<Text>,
        registration_mode -> RegistrationMode,
        application_question -> Nullable<Text>,
        is_site_setup -> Bool,
        is_private -> Bool,
        require_email_verification -> Bool,
        application_email_admins -> Bool,
        captcha_enabled -> Bool,
        #[max_length = 255]
        captcha_difficulty -> Varchar,
        enable_downvotes -> Bool,
        enable_nsfw -> Bool,
        enable_nsfw_tagging -> Bool,
        hide_modlog_mod_names -> Bool,
        reports_email_admins -> Bool,
        boards_enabled -> Bool,
        board_creation_admin_only -> Bool,
        #[max_length = 20]
        board_creation_mode -> Varchar,
        trusted_user_min_reputation -> Int4,
        trusted_user_min_account_age_days -> Int4,
        trusted_user_manual_approval -> Bool,
        trusted_user_min_posts -> Int4,
        allowed_post_types -> Nullable<Text>,
        word_filter_enabled -> Bool,
        filtered_words -> Nullable<Text>,
        word_filter_applies_to_posts -> Bool,
        word_filter_applies_to_comments -> Bool,
        word_filter_applies_to_usernames -> Bool,
        link_filter_enabled -> Bool,
        banned_domains -> Nullable<Text>,
        approved_image_hosts -> Nullable<Text>,
        image_embed_hosts_only -> Bool,
        emoji_enabled -> Bool,
        max_emojis_per_post -> Nullable<Int4>,
        max_emojis_per_comment -> Nullable<Int4>,
        emoji_max_file_size_mb -> Int4,
        board_emojis_enabled -> Bool,
        image_max_width -> Int4,
        image_max_height -> Int4,
        image_thumbnail_width -> Int4,
        image_convert_to_webp -> Bool,
        image_strip_exif -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    users (id) {
        id -> Uuid,
        #[max_length = 30]
        name -> Varchar,
        #[max_length = 30]
        display_name -> Nullable<Varchar>,
        email -> Nullable<Text>,
        passhash -> Text,
        is_email_verified -> Bool,
        is_banned -> Bool,
        is_admin -> Bool,
        admin_level -> Int4,
        is_bot_account -> Bool,
        is_board_creation_approved -> Bool,
        is_application_accepted -> Bool,
        unban_date -> Nullable<Timestamptz>,
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
        default_sort_type -> SortType,
        default_listing_type -> ListingType,
        interface_language -> Text,
        is_email_notifications_enabled -> Bool,
        editor_mode -> EditorMode,
        last_seen_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    boards (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 150]
        title -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 10000]
        sidebar -> Nullable<Varchar>,
        sidebar_html -> Nullable<Text>,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 25]
        primary_color -> Varchar,
        #[max_length = 25]
        secondary_color -> Varchar,
        #[max_length = 25]
        hover_color -> Varchar,
        is_nsfw -> Bool,
        is_hidden -> Bool,
        is_removed -> Bool,
        is_banned -> Bool,
        is_posting_restricted_to_mods -> Bool,
        exclude_from_all -> Bool,
        #[max_length = 512]
        ban_reason -> Nullable<Varchar>,
        public_ban_reason -> Nullable<Text>,
        banned_by -> Nullable<Uuid>,
        banned_at -> Nullable<Timestamptz>,
        section_config -> Int4,
        section_order -> Nullable<Text>,
        default_section -> Nullable<Text>,
        wiki_enabled -> Bool,
        wiki_require_approval -> Nullable<Bool>,
        wiki_default_view_permission -> WikiPermission,
        wiki_default_edit_permission -> WikiPermission,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    board_moderators (id) {
        id -> Uuid,
        board_id -> Uuid,
        user_id -> Uuid,
        permissions -> Int4,
        rank -> Int4,
        is_invite_accepted -> Bool,
        invite_accepted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    languages (id) {
        id -> Int4,
        #[max_length = 3]
        code -> Varchar,
        name -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    posts (id) {
        id -> Uuid,
        #[max_length = 200]
        title -> Varchar,
        post_type -> PostType,
        url -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        body -> Text,
        body_html -> Text,
        image -> Nullable<Text>,
        alt_text -> Nullable<Text>,
        #[max_length = 80]
        slug -> Varchar,
        creator_id -> Uuid,
        board_id -> Uuid,
        language_id -> Nullable<Int4>,
        is_removed -> Bool,
        is_locked -> Bool,
        is_nsfw -> Bool,
        is_featured_board -> Bool,
        is_featured_local -> Bool,
        approval_status -> ApprovalStatus,
        approved_by -> Nullable<Uuid>,
        approved_at -> Nullable<Timestamptz>,
        embed_title -> Nullable<Text>,
        embed_description -> Nullable<Text>,
        embed_video_url -> Nullable<Text>,
        source_url -> Nullable<Text>,
        last_crawl_date -> Nullable<Timestamptz>,
        is_thread -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    comments (id) {
        id -> Uuid,
        body -> Text,
        body_html -> Text,
        #[max_length = 80]
        slug -> Varchar,
        creator_id -> Uuid,
        post_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        board_id -> Uuid,
        language_id -> Nullable<Int4>,
        level -> Int4,
        is_removed -> Bool,
        is_locked -> Bool,
        is_read -> Bool,
        is_pinned -> Bool,
        approval_status -> ApprovalStatus,
        approved_by -> Nullable<Uuid>,
        approved_at -> Nullable<Timestamptz>,
        quoted_comment_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    post_votes (id) {
        id -> Uuid,
        user_id -> Uuid,
        post_id -> Uuid,
        score -> Int2,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    comment_votes (id) {
        id -> Uuid,
        user_id -> Uuid,
        comment_id -> Uuid,
        post_id -> Uuid,
        score -> Int2,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    post_aggregates (id) {
        id -> Uuid,
        post_id -> Uuid,
        board_id -> Uuid,
        creator_id -> Uuid,
        comments -> Int8,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        hot_rank -> Int4,
        hot_rank_active -> Int4,
        controversy_rank -> Float8,
        is_featured_board -> Bool,
        is_featured_local -> Bool,
        newest_comment_time -> Timestamptz,
        newest_comment_time_necro -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    comment_aggregates (id) {
        id -> Uuid,
        comment_id -> Uuid,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        child_count -> Int4,
        hot_rank -> Int4,
        controversy_rank -> Float8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    board_aggregates (id) {
        id -> Uuid,
        board_id -> Uuid,
        subscribers -> Int8,
        posts -> Int8,
        comments -> Int8,
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_aggregates (id) {
        id -> Uuid,
        user_id -> Uuid,
        post_count -> Int8,
        post_score -> Int8,
        comment_count -> Int8,
        comment_score -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    site_aggregates (id) {
        id -> Uuid,
        site_id -> Uuid,
        users -> Int8,
        posts -> Int8,
        comments -> Int8,
        boards -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    board_subscribers (id) {
        id -> Uuid,
        board_id -> Uuid,
        user_id -> Uuid,
        is_pending -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    board_user_bans (id) {
        id -> Uuid,
        board_id -> Uuid,
        user_id -> Uuid,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_bans (id) {
        id -> Uuid,
        user_id -> Uuid,
        banned_by -> Nullable<Uuid>,
        reason -> Nullable<Text>,
        expires_at -> Nullable<Timestamptz>,
        banned_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_blocks (id) {
        id -> Uuid,
        user_id -> Uuid,
        target_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    board_blocks (id) {
        id -> Uuid,
        user_id -> Uuid,
        board_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    post_saved (id) {
        id -> Uuid,
        post_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    comment_saved (id) {
        id -> Uuid,
        comment_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    post_hidden (id) {
        id -> Uuid,
        post_id -> Uuid,
        user_id -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_follows (id) {
        id -> Uuid,
        user_id -> Uuid,
        follower_id -> Uuid,
        is_pending -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    board_languages (id) {
        id -> Int4,
        board_id -> Uuid,
        language_id -> Int4,
    }
}

diesel::table! {
    site_languages (id) {
        id -> Int4,
        site_id -> Uuid,
        language_id -> Int4,
    }
}

diesel::table! {
    user_languages (id) {
        id -> Int4,
        user_id -> Uuid,
        language_id -> Int4,
    }
}

diesel::table! {
    private_messages (id) {
        id -> Uuid,
        creator_id -> Uuid,
        recipient_id -> Nullable<Uuid>,
        recipient_board_id -> Nullable<Uuid>,
        #[max_length = 200]
        subject -> Nullable<Varchar>,
        body -> Text,
        body_html -> Text,
        is_read -> Bool,
        is_sender_hidden -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    notifications (id) {
        id -> Uuid,
        kind -> NotificationKind,
        recipient_user_id -> Uuid,
        comment_id -> Nullable<Uuid>,
        post_id -> Nullable<Uuid>,
        message_id -> Nullable<Uuid>,
        is_read -> Bool,
        created_at -> Timestamptz,
        actor_user_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    notification_settings (id) {
        id -> Uuid,
        user_id -> Uuid,
        is_email_enabled -> Bool,
        is_comment_replies_enabled -> Bool,
        is_post_replies_enabled -> Bool,
        is_mentions_enabled -> Bool,
        is_private_messages_enabled -> Bool,
        is_board_invites_enabled -> Bool,
        is_moderator_actions_enabled -> Bool,
        is_system_notifications_enabled -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    moderation_log (id) {
        id -> Uuid,
        moderator_id -> Uuid,
        action_type -> ModerationAction,
        #[max_length = 20]
        target_type -> Varchar,
        target_id -> Uuid,
        board_id -> Nullable<Uuid>,
        reason -> Nullable<Text>,
        metadata -> Nullable<Jsonb>,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    post_reports (id) {
        id -> Uuid,
        creator_id -> Uuid,
        post_id -> Uuid,
        #[max_length = 200]
        original_post_title -> Varchar,
        original_post_url -> Nullable<Text>,
        original_post_body -> Nullable<Text>,
        reason -> Text,
        status -> ReportStatus,
        resolver_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    comment_reports (id) {
        id -> Uuid,
        creator_id -> Uuid,
        comment_id -> Uuid,
        original_comment_text -> Text,
        reason -> Text,
        status -> ReportStatus,
        resolver_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    uploads (id) {
        id -> Uuid,
        user_id -> Uuid,
        original_name -> Text,
        file_name -> Text,
        file_path -> Text,
        upload_url -> Text,
        size_bytes -> Int8,
        created_at -> Timestamptz,
        thumbnail_url -> Nullable<Text>,
        optimized_url -> Nullable<Text>,
        #[max_length = 20]
        processing_status -> Varchar,
    }
}

diesel::table! {
    content_uploads (id) {
        id -> Uuid,
        upload_id -> Uuid,
        post_id -> Nullable<Uuid>,
        comment_id -> Nullable<Uuid>,
        position -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    emoji (id) {
        id -> Uuid,
        #[max_length = 128]
        shortcode -> Varchar,
        image_url -> Text,
        alt_text -> Text,
        category -> Text,
        scope -> EmojiScope,
        board_id -> Nullable<Uuid>,
        created_by -> Uuid,
        is_active -> Bool,
        usage_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    emoji_keywords (id) {
        id -> Uuid,
        emoji_id -> Uuid,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::table! {
    reactions (id) {
        id -> Uuid,
        user_id -> Uuid,
        post_id -> Nullable<Uuid>,
        comment_id -> Nullable<Uuid>,
        #[max_length = 100]
        emoji -> Varchar,
        score -> Int4,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    reaction_aggregates (id) {
        id -> Uuid,
        post_id -> Nullable<Uuid>,
        comment_id -> Nullable<Uuid>,
        #[max_length = 100]
        emoji -> Varchar,
        count -> Int4,
    }
}

diesel::table! {
    board_reaction_settings (id) {
        id -> Uuid,
        board_id -> Uuid,
        emoji_weights -> Jsonb,
        is_reactions_enabled -> Bool,
    }
}

diesel::table! {
    flair_categories (id) {
        id -> Uuid,
        board_id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 7]
        color -> Nullable<Varchar>,
        display_order -> Int4,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    flair_templates (id) {
        id -> Uuid,
        board_id -> Uuid,
        flair_type -> FlairType,
        #[max_length = 100]
        template_name -> Varchar,
        #[max_length = 50]
        template_key -> Nullable<Varchar>,
        #[max_length = 64]
        text_display -> Varchar,
        #[max_length = 7]
        text_color -> Varchar,
        #[max_length = 7]
        background_color -> Varchar,
        style_config -> Jsonb,
        emoji_ids -> Array<Nullable<Int4>>,
        is_mod_only -> Bool,
        is_editable -> Bool,
        max_emoji_count -> Int4,
        max_text_length -> Int4,
        is_requires_approval -> Bool,
        display_order -> Int4,
        is_active -> Bool,
        usage_count -> Int4,
        category_id -> Nullable<Uuid>,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    post_flairs (id) {
        id -> Uuid,
        post_id -> Uuid,
        flair_template_id -> Uuid,
        #[max_length = 64]
        custom_text -> Nullable<Varchar>,
        #[max_length = 7]
        custom_text_color -> Nullable<Varchar>,
        #[max_length = 7]
        custom_background_color -> Nullable<Varchar>,
        assigned_by -> Uuid,
        is_original_author -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    user_flairs (id) {
        id -> Uuid,
        user_id -> Uuid,
        board_id -> Uuid,
        flair_template_id -> Uuid,
        #[max_length = 64]
        custom_text -> Nullable<Varchar>,
        #[max_length = 7]
        custom_text_color -> Nullable<Varchar>,
        #[max_length = 7]
        custom_background_color -> Nullable<Varchar>,
        is_approved -> Bool,
        approved_at -> Nullable<Timestamptz>,
        approved_by -> Nullable<Uuid>,
        is_self_assigned -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    user_flair_filters (id) {
        id -> Uuid,
        user_id -> Uuid,
        board_id -> Uuid,
        filter_mode -> FilterMode,
        included_flair_ids -> Array<Nullable<Int4>>,
        excluded_flair_ids -> Array<Nullable<Int4>>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    flair_aggregates (id) {
        id -> Uuid,
        flair_template_id -> Uuid,
        total_usage_count -> Int4,
        post_usage_count -> Int4,
        user_usage_count -> Int4,
        active_user_count -> Int4,
        usage_last_day -> Int4,
        usage_last_week -> Int4,
        usage_last_month -> Int4,
        avg_post_score -> Numeric,
        total_post_comments -> Int4,
        total_post_score -> Int4,
        trending_score -> Numeric,
        hot_rank -> Numeric,
        last_used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::*;

    wiki_pages (id) {
        id -> Uuid,
        board_id -> Uuid,
        #[max_length = 100]
        slug -> Varchar,
        #[max_length = 200]
        title -> Varchar,
        body -> Text,
        body_html -> Text,
        creator_id -> Uuid,
        last_edited_by -> Nullable<Uuid>,
        view_permission -> WikiPermission,
        edit_permission -> WikiPermission,
        is_locked -> Bool,
        display_order -> Nullable<Int4>,
        parent_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    wiki_page_revisions (id) {
        id -> Uuid,
        page_id -> Uuid,
        revision_number -> Int4,
        editor_id -> Uuid,
        edit_summary -> Nullable<Text>,
        body -> Text,
        body_html -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    wiki_approved_contributors (id) {
        id -> Uuid,
        board_id -> Uuid,
        user_id -> Uuid,
        added_by -> Uuid,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    secrets (id) {
        id -> Uuid,
        jwt_secret -> Varchar,
    }
}

diesel::table! {
    password_resets (id) {
        id -> Uuid,
        user_id -> Uuid,
        reset_token -> Text,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        used_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    email_verification (id) {
        id -> Uuid,
        user_id -> Uuid,
        email -> Text,
        verification_code -> Text,
        created_at -> Timestamptz,
        verified_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    registration_applications (id) {
        id -> Uuid,
        user_id -> Uuid,
        answer -> Text,
        admin_id -> Nullable<Uuid>,
        deny_reason -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    site_invites (id) {
        id -> Uuid,
        verification_code -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    rate_limits (id) {
        id -> Uuid,
        site_id -> Uuid,
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
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    auth_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        refresh_token_hash -> Text,
        user_agent -> Nullable<Text>,
        ip_address -> Nullable<Text>,
        last_used_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

// ============================================================
// Joinable declarations (FK relationships for Diesel joins)
// ============================================================

diesel::joinable!(auth_sessions -> users (user_id));
diesel::joinable!(board_aggregates -> boards (board_id));
diesel::joinable!(board_blocks -> boards (board_id));
diesel::joinable!(board_blocks -> users (user_id));
diesel::joinable!(board_languages -> boards (board_id));
diesel::joinable!(board_languages -> languages (language_id));
diesel::joinable!(board_moderators -> boards (board_id));
diesel::joinable!(board_moderators -> users (user_id));
diesel::joinable!(board_reaction_settings -> boards (board_id));
diesel::joinable!(board_subscribers -> boards (board_id));
diesel::joinable!(board_subscribers -> users (user_id));
diesel::joinable!(board_user_bans -> boards (board_id));
diesel::joinable!(board_user_bans -> users (user_id));
diesel::joinable!(comment_aggregates -> comments (comment_id));
diesel::joinable!(comment_reports -> comments (comment_id));
diesel::joinable!(comment_saved -> comments (comment_id));
diesel::joinable!(comment_saved -> users (user_id));
diesel::joinable!(comment_votes -> comments (comment_id));
diesel::joinable!(comment_votes -> posts (post_id));
diesel::joinable!(comment_votes -> users (user_id));
diesel::joinable!(comments -> languages (language_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(content_uploads -> comments (comment_id));
diesel::joinable!(content_uploads -> posts (post_id));
diesel::joinable!(content_uploads -> uploads (upload_id));
diesel::joinable!(email_verification -> users (user_id));
diesel::joinable!(emoji_keywords -> emoji (emoji_id));
diesel::joinable!(flair_aggregates -> flair_templates (flair_template_id));
diesel::joinable!(flair_categories -> boards (board_id));
diesel::joinable!(flair_templates -> boards (board_id));
diesel::joinable!(flair_templates -> flair_categories (category_id));
diesel::joinable!(moderation_log -> boards (board_id));
diesel::joinable!(notification_settings -> users (user_id));
diesel::joinable!(notifications -> private_messages (message_id));
diesel::joinable!(password_resets -> users (user_id));
diesel::joinable!(post_aggregates -> posts (post_id));
diesel::joinable!(post_flairs -> flair_templates (flair_template_id));
diesel::joinable!(post_flairs -> posts (post_id));
diesel::joinable!(post_hidden -> posts (post_id));
diesel::joinable!(post_hidden -> users (user_id));
diesel::joinable!(post_reports -> posts (post_id));
diesel::joinable!(post_saved -> posts (post_id));
diesel::joinable!(post_saved -> users (user_id));
diesel::joinable!(post_votes -> posts (post_id));
diesel::joinable!(post_votes -> users (user_id));
diesel::joinable!(posts -> boards (board_id));
diesel::joinable!(posts -> languages (language_id));
diesel::joinable!(rate_limits -> site (site_id));
diesel::joinable!(reaction_aggregates -> comments (comment_id));
diesel::joinable!(reaction_aggregates -> posts (post_id));
diesel::joinable!(reactions -> comments (comment_id));
diesel::joinable!(reactions -> posts (post_id));
diesel::joinable!(reactions -> users (user_id));
diesel::joinable!(registration_applications -> users (user_id));
diesel::joinable!(site_aggregates -> site (site_id));
diesel::joinable!(site_languages -> languages (language_id));
diesel::joinable!(site_languages -> site (site_id));
diesel::joinable!(uploads -> users (user_id));
diesel::joinable!(user_aggregates -> users (user_id));
diesel::joinable!(user_flair_filters -> boards (board_id));
diesel::joinable!(user_flair_filters -> users (user_id));
diesel::joinable!(user_flairs -> boards (board_id));
diesel::joinable!(user_flairs -> flair_templates (flair_template_id));
diesel::joinable!(user_languages -> languages (language_id));
diesel::joinable!(user_languages -> users (user_id));
diesel::joinable!(wiki_approved_contributors -> boards (board_id));
diesel::joinable!(wiki_page_revisions -> users (editor_id));
diesel::joinable!(wiki_page_revisions -> wiki_pages (page_id));
diesel::joinable!(wiki_pages -> boards (board_id));

// ============================================================
// Allow all tables in same query
// ============================================================

diesel::allow_tables_to_appear_in_same_query!(
    auth_sessions,
    board_aggregates,
    board_blocks,
    board_languages,
    board_moderators,
    board_reaction_settings,
    board_subscribers,
    board_user_bans,
    boards,
    comment_aggregates,
    comment_reports,
    comment_saved,
    comment_votes,
    comments,
    content_uploads,
    email_verification,
    emoji,
    emoji_keywords,
    flair_aggregates,
    flair_categories,
    flair_templates,
    languages,
    moderation_log,
    notification_settings,
    notifications,
    password_resets,
    post_aggregates,
    post_flairs,
    post_hidden,
    post_reports,
    post_saved,
    post_votes,
    posts,
    private_messages,
    rate_limits,
    reaction_aggregates,
    reactions,
    registration_applications,
    secrets,
    site,
    site_aggregates,
    site_invites,
    site_languages,
    uploads,
    user_aggregates,
    user_bans,
    user_blocks,
    user_flair_filters,
    user_flairs,
    user_follows,
    user_languages,
    users,
    wiki_approved_contributors,
    wiki_page_revisions,
    wiki_pages,
);
