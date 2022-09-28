// @generated automatically by Diesel CLI.

diesel::table! {
    alts (id) {
        id -> Int4,
        user1 -> Int4,
        user2 -> Int4,
        is_manual -> Bool,
    }
}

diesel::table! {
    badge_defs (id) {
        id -> Int4,
        badge_name -> Varchar,
        badge_description -> Varchar,
        badge_icon -> Varchar,
        badge_kind -> Int2,
        badge_rank -> Int2,
        qualification_expr -> Nullable<Varchar>,
    }
}

diesel::table! {
    badges (id) {
        id -> Int4,
        user_id -> Int4,
        badge_id -> Int4,
        badge_description -> Varchar,
        badge_url -> Varchar,
        created_utc -> Int4,
    }
}

diesel::table! {
    badlinks (id) {
        id -> Int4,
        reason -> Int4,
        link -> Varchar,
        autoban -> Bool,
    }
}

diesel::table! {
    badpics (id) {
        id -> Int4,
        badpic_description -> Nullable<Varchar>,
        phash -> Varchar,
        ban_reason -> Varchar,
        ban_time -> Int4,
    }
}

diesel::table! {
    badwords (id) {
        id -> Int4,
        keyword -> Varchar,
        regex -> Varchar,
    }
}

diesel::table! {
    bans (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
        banning_mod_id -> Int4,
        is_active -> Int4,
        mod_note -> Varchar,
    }
}

diesel::table! {
    boardblocks (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
    }
}

diesel::table! {
    boards (id) {
        id -> Int4,
        board_name -> Varchar,
        created_utc -> Int4,
        board_description -> Nullable<Varchar>,
        board_description_html -> Nullable<Varchar>,
        over_18 -> Bool,
        is_nsfl -> Bool,
        is_banned -> Bool,
        has_banner -> Bool,
        has_profile -> Bool,
        creator_id -> Int4,
        ban_reason -> Nullable<Varchar>,
        color -> Nullable<Varchar>,
        restricted_posting -> Bool,
        disallowbots -> Bool,
        hide_banner_data -> Bool,
        profile_nonce -> Int4,
        banner_nonce -> Int4,
        is_private -> Bool,
        color_nonce -> Int4,
        rank_trending -> Numeric,
        stored_subscriber_count -> Int4,
        all_opt_out -> Bool,
        is_locked_category -> Bool,
        subcat_id -> Int4,
        secondary_color -> Varchar,
        public_chat -> Bool,
        motd -> Varchar,
        css_nonce -> Int4,
        css -> Varchar,
    }
}

diesel::table! {
    categories (id) {
        id -> Int4,
        category_name -> Varchar,
        category_description -> Varchar,
        category_icon -> Varchar,
        category_color -> Nullable<Varchar>,
        visible -> Bool,
        is_nsfw -> Bool,
    }
}

diesel::table! {
    chatbans (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
        banning_mod_id -> Int4,
    }
}

diesel::table! {
    client_auths (id) {
        id -> Int4,
        oauth_client -> Int4,
        oauth_code -> Varchar,
        user_id -> Int4,
        scope_identity -> Bool,
        scope_create -> Bool,
        scope_read -> Bool,
        scope_update -> Bool,
        scope_delete -> Bool,
        scope_vote -> Bool,
        scope_moderator -> Bool,
        access_token -> Varchar,
        refresh_token -> Varchar,
        access_token_expire_utc -> Int4,
    }
}

diesel::table! {
    commentflags (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        created_utc -> Int4,
    }
}

diesel::table! {
    comments (id) {
        id -> Int4,
        body -> Nullable<Varchar>,
        body_html -> Nullable<Varchar>,
        ban_reason -> Varchar,
        author_id -> Int4,
        parent_submission -> Int4,
        created_utc -> Int4,
        edited_utc -> Int4,
        is_banned -> Bool,
        gm_distinguish -> Int4,
        distinguished_board -> Nullable<Int4>,
        deleted_utc -> Int4,
        purged_utc -> Int4,
        is_approved -> Int4,
        approved_utc -> Int4,
        creation_ip -> Varchar,
        score_disputed -> Numeric,
        score_hot -> Numeric,
        score_top -> Numeric,
        comment_level -> Int4,
        parent_comment_id -> Int4,
        original_board_id -> Int4,
        over_18 -> Bool,
        is_offensive -> Bool,
        is_nsfl -> Bool,
        is_bot -> Bool,
        is_pinned -> Bool,
        creation_region -> Nullable<Varchar>,
        app_id -> Nullable<Int4>,
        upvotes -> Int4,
        downvotes -> Int4,
    }
}

diesel::table! {
    commentvotes (id) {
        id -> Int8,
        user_id -> Int4,
        vote_type -> Int4,
        comment_id -> Int4,
        created_utc -> Int4,
        creation_ip -> Varchar,
        app_id -> Nullable<Int4>,
    }
}

diesel::table! {
    contributors (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
        is_active -> Bool,
        approving_mod_id -> Int4,
    }
}

diesel::table! {
    domains (id) {
        id -> Int4,
        domain -> Varchar,
        can_submit -> Bool,
        can_comment -> Bool,
        reason -> Int4,
        show_thumbnail -> Bool,
        embed_function -> Nullable<Varchar>,
        embed_template -> Nullable<Varchar>,
    }
}

diesel::table! {
    flags (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        created_utc -> Int4,
    }
}

diesel::table! {
    follows (id) {
        id -> Int4,
        user_id -> Int4,
        target_id -> Int4,
        created_utc -> Int4,
        get_notifs -> Bool,
    }
}

diesel::table! {
    images (id) {
        id -> Int4,
        img_state -> Nullable<Varchar>,
        img_number -> Nullable<Int4>,
        img_text -> Nullable<Varchar>,
    }
}

diesel::table! {
    ips (id) {
        id -> Int4,
        addr -> Varchar,
        reason -> Varchar,
        banned_by -> Int4,
    }
}

diesel::table! {
    lodges (id) {
        id -> Int4,
        lodge_name -> Varchar,
        lodge_color -> Varchar,
        lodge_description -> Varchar,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
    }
}

diesel::table! {
    modactions (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        kind -> Varchar,
        target_user_id -> Int4,
        target_submission_id -> Int4,
        target_comment_id -> Int4,
        note -> Nullable<Varchar>,
        created_utc -> Int4,
    }
}

diesel::table! {
    mods (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
        accepted -> Bool,
        invite_rescinded -> Bool,
        perm_content -> Bool,
        perm_appearance -> Bool,
        perm_config -> Bool,
        perm_access -> Bool,
        perm_full -> Bool,
    }
}

diesel::table! {
    notifications (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Nullable<Int4>,
        submission_id -> Nullable<Int4>,
        notification_read -> Bool,
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
        is_banned -> Bool,
        app_description -> Varchar,
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
    reports (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        created_utc -> Int4,
    }
}

diesel::table! {
    rules (id) {
        id -> Int4,
        board_id -> Int4,
        rule_body -> Varchar,
        rule_html -> Varchar,
        created_utc -> Int4,
        edited_utc -> Int4,
    }
}

diesel::table! {
    save_relationship (id) {
        id -> Int4,
        user_id -> Int4,
        submission_id -> Int4,
    }
}

diesel::table! {
    subcategories (id) {
        id -> Int4,
        cat_id -> Int4,
        subcat_name -> Varchar,
        subcat_description -> Varchar,
        _visible -> Bool,
    }
}

diesel::table! {
    submissions (id) {
        id -> Int4,
        title -> Nullable<Varchar>,
        post_url -> Nullable<Varchar>,
        body -> Varchar,
        ban_reason -> Varchar,
        embed_url -> Varchar,
        meta_title -> Varchar,
        meta_description -> Varchar,
        author_id -> Int4,
        repost_id -> Int4,
        edited_utc -> Int4,
        created_utc -> Int4,
        is_banned -> Bool,
        deleted_utc -> Int4,
        distinguish_level -> Int2,
        gm_distinguish -> Int2,
        stickied -> Bool,
        is_approved -> Int4,
        approved_utc -> Int4,
        board_id -> Int4,
        original_board_id -> Int4,
        over_18 -> Bool,
        creation_ip -> Varchar,
        mod_approved -> Nullable<Int4>,
        accepted_utc -> Int4,
        has_thumb -> Bool,
        post_public -> Bool,
        score_hot -> Numeric,
        score_disputed -> Numeric,
        score_top -> Numeric,
        score_best -> Numeric,
        score_activity -> Numeric,
        is_offensive -> Bool,
        is_nsfl -> Bool,
        is_pinned -> Bool,
        is_bot -> Bool,
        upvotes -> Int4,
        downvotes -> Int4,
        creation_region -> Nullable<Varchar>,
        app_id -> Nullable<Int4>,
    }
}

diesel::table! {
    subscriptions (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
        created_utc -> Int4,
        is_active -> Bool,
        get_notifs -> Bool,
    }
}

diesel::table! {
    titles (id) {
        id -> Int4,
        is_before -> Bool,
        title_text -> Varchar,
        qualification_expr -> Varchar,
        requirement_string -> Varchar,
        title_color -> Varchar,
        bg_color_1 -> Nullable<Varchar>,
        bg_color_2 -> Nullable<Varchar>,
        gradient_angle -> Int4,
        box_shadow_color -> Nullable<Varchar>,
        text_shadow_color -> Nullable<Varchar>,
    }
}

diesel::table! {
    useragents (id) {
        id -> Int4,
        kwd -> Varchar,
        reason -> Varchar,
        banned_by -> Int4,
        mock -> Varchar,
        status_code -> Int4,
    }
}

diesel::table! {
    userblocks (id) {
        id -> Int4,
        user_id -> Int4,
        target_id -> Int4,
        created_utc -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Nullable<Varchar>,
        passhash -> Varchar,
        created_utc -> Int4,
        admin_level -> Int2,
        is_activated -> Bool,
        over_18 -> Bool,
        creation_ip -> Varchar,
        bio -> Varchar,
        bio_html -> Varchar,
        referred_by -> Nullable<Int4>,
        is_banned -> Bool,
        unban_utc -> Int4,
        ban_reason -> Varchar,
        defaultsorting -> Nullable<Varchar>,
        defaulttime -> Nullable<Varchar>,
        feed_nonce -> Int4,
        login_nonce -> Int4,
        title_id -> Nullable<Int4>,
        has_profile -> Bool,
        has_banner -> Bool,
        reserved -> Nullable<Varchar>,
        is_nsfw -> Bool,
        tos_agreed_utc -> Int4,
        profile_nonce -> Int4,
        banner_nonce -> Int4,
        mfa_secret -> Nullable<Varchar>,
        hide_offensive -> Bool,
        hide_bot -> Bool,
        show_nsfl -> Bool,
        is_private -> Bool,
        is_deleted -> Bool,
        delete_reason -> Varchar,
        filter_nsfw -> Bool,
        stored_karma -> Int4,
        stored_subscriber_count -> Int4,
        auto_join_chat -> Bool,
        is_nofollow -> Bool,
        custom_filter_list -> Varchar,
        discord_id -> Nullable<Varchar>,
        creation_region -> Nullable<Varchar>,
        ban_evade -> Int4,
        profile_upload_ip -> Varchar,
        banner_upload_ip -> Varchar,
        profile_upload_region -> Varchar,
        banner_upload_region -> Varchar,
        color -> Varchar,
        secondary_color -> Varchar,
        comment_signature -> Varchar,
        comment_signature_html -> Varchar,
        profile_set_utc -> Int4,
        bannner_set_utc -> Int4,
        original_username -> Varchar,
        name_changed_utc -> Int4,
    }
}

diesel::table! {
    votes (id) {
        id -> Int8,
        user_id -> Int4,
        vote_type -> Int4,
        submission_id -> Int4,
        created_utc -> Int4,
        creation_ip -> Varchar,
        app_id -> Nullable<Int4>,
    }
}

diesel::joinable!(badges -> badge_defs (badge_id));
diesel::joinable!(boards -> subcategories (subcat_id));
diesel::joinable!(boards -> users (creator_id));
diesel::joinable!(client_auths -> oauth_apps (oauth_client));
diesel::joinable!(comments -> submissions (parent_submission));
diesel::joinable!(contributors -> users (approving_mod_id));
diesel::joinable!(modactions -> comments (target_comment_id));
diesel::joinable!(modactions -> submissions (target_submission_id));
diesel::joinable!(modactions -> users (target_user_id));
diesel::joinable!(subcategories -> categories (cat_id));
diesel::joinable!(users -> titles (title_id));

diesel::allow_tables_to_appear_in_same_query!(
    alts,
    badge_defs,
    badges,
    badlinks,
    badpics,
    badwords,
    bans,
    boardblocks,
    boards,
    categories,
    chatbans,
    client_auths,
    commentflags,
    comments,
    commentvotes,
    contributors,
    domains,
    flags,
    follows,
    images,
    ips,
    lodges,
    modactions,
    mods,
    notifications,
    oauth_apps,
    postrels,
    reports,
    rules,
    save_relationship,
    subcategories,
    submissions,
    subscriptions,
    titles,
    useragents,
    userblocks,
    users,
    votes,
);
