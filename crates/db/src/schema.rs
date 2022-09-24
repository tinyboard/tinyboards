// @generated automatically by Diesel CLI.

diesel::table! {
    alts (id) {
        id -> Int4,
        user1 -> Int4,
        user2 -> Int4,
        is_manual -> Nullable<Bool>,
    }
}

diesel::table! {
    badge_defs (id) {
        id -> Int4,
        badge_name -> Nullable<Varchar>,
        badge_description -> Nullable<Varchar>,
        badge_icon -> Nullable<Varchar>,
        badge_kind -> Nullable<Int2>,
        badge_rank -> Nullable<Int2>,
        qualification_expr -> Nullable<Varchar>,
    }
}

diesel::table! {
    badges (id) {
        id -> Int4,
        user_id -> Int4,
        badge_id -> Int4,
        badge_description -> Nullable<Varchar>,
        badge_url -> Nullable<Varchar>,
        created_utc -> Nullable<Int4>,
    }
}

diesel::table! {
    badlinks (id) {
        id -> Int4,
        reason -> Int4,
        link -> Varchar,
        autoban -> Nullable<Bool>,
    }
}

diesel::table! {
    badwords (id) {
        id -> Int4,
        keyword -> Nullable<Varchar>,
        regex -> Nullable<Varchar>,
    }
}

diesel::table! {
    bans (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Nullable<Int4>,
        banning_mod_id -> Int4,
        is_active -> Int4,
        mod_note -> Nullable<Varchar>,
    }
}

diesel::table! {
    boardblocks (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Nullable<Int4>,
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
        css_nonce -> Nullable<Int4>,
        css -> Nullable<Varchar>,
    }
}

diesel::table! {
    categories (id) {
        id -> Int4,
        category_name -> Nullable<Varchar>,
        category_description -> Nullable<Varchar>,
        category_icon -> Nullable<Varchar>,
        category_color -> Nullable<Varchar>,
        visible -> Nullable<Bool>,
        is_nsfw -> Nullable<Bool>,
    }
}

diesel::table! {
    chatbans (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Nullable<Int4>,
        banning_mod_id -> Int4,
    }
}

diesel::table! {
    client_auths (id) {
        id -> Int4,
        oauth_client -> Int4,
        oauth_code -> Nullable<Varchar>,
        user_id -> Int4,
        scope_identity -> Nullable<Bool>,
        scope_create -> Nullable<Bool>,
        scope_read -> Nullable<Bool>,
        scope_update -> Nullable<Bool>,
        scope_delete -> Nullable<Bool>,
        scope_vote -> Nullable<Bool>,
        scope_moderator -> Nullable<Bool>,
        access_token -> Nullable<Varchar>,
        refresh_token -> Nullable<Varchar>,
        access_token_expire_utc -> Nullable<Int4>,
    }
}

diesel::table! {
    contributors (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
        is_active -> Nullable<Bool>,
        approving_mod_id -> Int4,
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
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Nullable<Int4>,
        accepted -> Nullable<Bool>,
        invite_rescinded -> Nullable<Bool>,
        perm_content -> Nullable<Bool>,
        perm_appearance -> Nullable<Bool>,
        perm_config -> Nullable<Bool>,
        perm_access -> Nullable<Bool>,
        perm_full -> Nullable<Bool>,
    }
}

diesel::table! {
    oauth_apps (id) {
        id -> Int4,
        client_id -> Varchar,
        client_secret -> Varchar,
        app_name -> Varchar,
        redirect_uri -> Varchar,
        author_id -> Int4,
        is_banned -> Nullable<Bool>,
        app_description -> Nullable<Varchar>,
    }
}

diesel::table! {
    postrels (id) {
        id -> Int8,
        post_id -> Int4,
        board_id -> Int4,
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
        cat_id -> Int4,
        subcat_name -> Nullable<Varchar>,
        subcat_description -> Nullable<Varchar>,
        _visible -> Nullable<Bool>,
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
    alts,
    badge_defs,
    badges,
    badlinks,
    badwords,
    bans,
    boardblocks,
    boards,
    categories,
    chatbans,
    client_auths,
    contributors,
    domains,
    mods,
    oauth_apps,
    postrels,
    posts,
    subcategories,
    submissions,
    users,
);
