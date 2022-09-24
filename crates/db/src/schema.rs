// @generated automatically by Diesel CLI.

diesel::table! {
    badlinks (id) {
        id -> Int4,
        reason -> Int4,
        link -> Varchar,
        autoban -> Nullable<Bool>,
    }
}

diesel::table! {
    boards (id) {
        id -> Int4,
        board_name -> Varchar,
        created_utc -> Nullable<Int4>,
        board_description -> Nullable<Varchar>,
        board_description_html -> Nullable<Varchar>,
        over_18 -> Nullable<Bool>,
        is_nsfl -> Nullable<Bool>,
        is_banned -> Nullable<Bool>,
        has_banner -> Nullable<Bool>,
        has_profile -> Nullable<Bool>,
        creator_id -> Int4,
        ban_reason -> Nullable<Varchar>,
        color -> Nullable<Varchar>,
        restricted_posting -> Nullable<Bool>,
        disallowbots -> Nullable<Bool>,
        hide_banner_data -> Nullable<Bool>,
        profile_nonce -> Nullable<Int4>,
        banner_nonce -> Nullable<Int4>,
        is_private -> Nullable<Bool>,
        color_nonce -> Nullable<Int4>,
        rank_trending -> Nullable<Numeric>,
        stored_subscriber_count -> Nullable<Int4>,
        all_opt_out -> Nullable<Bool>,
        is_locked_category -> Nullable<Bool>,
        subcat_id -> Nullable<Int4>,
        secondary_color -> Nullable<Varchar>,
        public_chat -> Nullable<Bool>,
        motd -> Nullable<Varchar>,
    }
}

diesel::table! {
    domains (id) {
        id -> Int4,
        domain -> Varchar,
        can_submit -> Nullable<Bool>,
        can_comment -> Nullable<Bool>,
        reason -> Nullable<Int4>,
        show_thumbnail -> Nullable<Bool>,
        embed_function -> Nullable<Varchar>,
        embed_template -> Nullable<Varchar>,
    }
}

diesel::table! {
    mods (id) {
        id -> Int4,
    }
}

diesel::table! {
    oauth_apps (id) {
        id -> Int4,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
    }
}

diesel::table! {
    subcategories (id) {
        id -> Int4,
    }
}

diesel::table! {
    submissions (id) {
        id -> Int8,
        author_id -> Int4,
        repost_id -> Nullable<Int4>,
        edited_utc -> Nullable<Int4>,
        created_utc -> Nullable<Int4>,
        is_banned -> Nullable<Bool>,
        deleted_utc -> Nullable<Int4>,
        purged_utc -> Nullable<Int4>,
        distinguish_level -> Nullable<Int2>,
        gm_distinguish -> Nullable<Int2>,
        created_str -> Nullable<Varchar>,
        stickied -> Nullable<Bool>,
        domain_ref -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        passhash -> Varchar,
        created_utc -> Int4,
        admin_level -> Int2,
        is_activated -> Bool,
        over_18 -> Bool,
        creation_ip -> Varchar,
        bio -> Nullable<Varchar>,
        bio_html -> Nullable<Varchar>,
        referred_by -> Nullable<Int4>,
        is_banned -> Nullable<Bool>,
        unban_utc -> Nullable<Int4>,
        ban_reason -> Nullable<Varchar>,
        defaultsorting -> Nullable<Varchar>,
        defaulttime -> Nullable<Varchar>,
        feed_nonce -> Nullable<Int4>,
        login_nonce -> Nullable<Int4>,
        has_profile -> Nullable<Bool>,
        has_banner -> Nullable<Bool>,
        reserved -> Nullable<Varchar>,
        is_nsfw -> Nullable<Bool>,
        tos_agreed_utc -> Nullable<Int4>,
        profile_nonce -> Nullable<Int4>,
        banner_nonce -> Nullable<Int4>,
        mfa_secret -> Nullable<Varchar>,
        hide_offensive -> Nullable<Bool>,
        hide_bot -> Nullable<Bool>,
        show_nsfl -> Nullable<Bool>,
        is_private -> Nullable<Bool>,
        is_deleted -> Nullable<Bool>,
        delete_reason -> Nullable<Varchar>,
        filter_nsfw -> Nullable<Bool>,
        stored_karma -> Nullable<Int4>,
        stored_subscriber_count -> Nullable<Int4>,
        auto_join_chat -> Nullable<Bool>,
        is_nofollow -> Nullable<Bool>,
        custom_filter_list -> Nullable<Varchar>,
        discord_id -> Nullable<Varchar>,
        creation_region -> Nullable<Varchar>,
        ban_evade -> Nullable<Int4>,
        profile_upload_ip -> Nullable<Varchar>,
        banner_upload_ip -> Nullable<Varchar>,
        profile_upload_region -> Nullable<Varchar>,
        banner_upload_region -> Nullable<Varchar>,
        color -> Nullable<Varchar>,
        secondary_color -> Nullable<Varchar>,
        comment_signature -> Nullable<Varchar>,
        comment_signature_html -> Nullable<Varchar>,
        profile_set_utc -> Nullable<Int4>,
        bannner_set_utc -> Nullable<Int4>,
        original_username -> Nullable<Varchar>,
        name_changed_utc -> Nullable<Int4>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    badlinks,
    boards,
    domains,
    mods,
    oauth_apps,
    posts,
    subcategories,
    submissions,
    users,
);
