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
    board_block (id) {
        id -> Int4,
        user_id -> Int4,
        board_id -> Int4,
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
        pending -> Nullable<Bool>,
    }
}

diesel::table! {
    board_user_ban (id) {
        id -> Int4,
        board_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
        expires -> Nullable<Timestamp>,
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
    postrels (id) {
        id -> Int8,
        post_id -> Int4,
        board_id -> Int4,
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
    registration_application (id) {
        id -> Int4,
        user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published -> Timestamp,
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
    tag (id) {
        id -> Int4,
        name -> Varchar,
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
        accepted_application -> Bool,
        deleted -> Bool,
        expires -> Nullable<Timestamp>,
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
    user_block (id) {
        id -> Int4,
        user_id -> Int4,
        target_id -> Int4,
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
        email -> Varchar,
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
diesel::joinable!(board -> tag (tag_id));
diesel::joinable!(board -> user_ (creator_id));
diesel::joinable!(board_aggregates -> board (board_id));
diesel::joinable!(board_block -> board (board_id));
diesel::joinable!(board_block -> user_ (user_id));
diesel::joinable!(board_moderator -> board (board_id));
diesel::joinable!(board_moderator -> user_ (user_id));
diesel::joinable!(board_subscriber -> board (board_id));
diesel::joinable!(board_subscriber -> user_ (user_id));
diesel::joinable!(board_user_ban -> board (board_id));
diesel::joinable!(board_user_ban -> user_ (user_id));
diesel::joinable!(boards -> subcategories (subcat_id));
diesel::joinable!(boards -> users (creator_id));
diesel::joinable!(client_auths -> oauth_apps (oauth_client));
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment -> user_ (creator_id));
diesel::joinable!(comment_aggregates -> comment (comment_id));
diesel::joinable!(comment_like -> comment (comment_id));
diesel::joinable!(comment_like -> post (post_id));
diesel::joinable!(comment_like -> user_ (user_id));
diesel::joinable!(comment_saved -> comment (comment_id));
diesel::joinable!(comment_saved -> user_ (user_id));
diesel::joinable!(comments -> submissions (parent_submission));
diesel::joinable!(contributors -> users (approving_mod_id));
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
diesel::joinable!(modactions -> comments (target_comment_id));
diesel::joinable!(modactions -> submissions (target_submission_id));
diesel::joinable!(modactions -> users (target_user_id));
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
diesel::joinable!(subcategories -> categories (cat_id));
diesel::joinable!(user_aggregates -> user_ (user_id));
diesel::joinable!(user_ban -> user_ (user_id));
diesel::joinable!(user_mention -> comment (comment_id));
diesel::joinable!(user_mention -> user_ (recipient_id));
diesel::joinable!(users -> titles (title_id));

diesel::allow_tables_to_appear_in_same_query!(
    alts,
    badge_defs,
    badges,
    badlinks,
    badpics,
    badwords,
    bans,
    board,
    board_aggregates,
    board_block,
    board_moderator,
    board_subscriber,
    board_user_ban,
    boardblocks,
    boards,
    categories,
    chatbans,
    client_auths,
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
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
    mod_add,
    mod_add_board,
    mod_ban,
    mod_ban_from_board,
    mod_lock_post,
    mod_remove_board,
    mod_remove_comment,
    mod_remove_post,
    mod_sticky_post,
    modactions,
    mods,
    notifications,
    oauth_apps,
    password_reset_request,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_saved,
    postrels,
    private_message,
    registration_application,
    reports,
    rules,
    save_relationship,
    site,
    subcategories,
    submissions,
    subscriptions,
    tag,
    titles,
    user_,
    user_aggregates,
    user_ban,
    user_block,
    user_mention,
    useragents,
    userblocks,
    users,
    votes,
);
