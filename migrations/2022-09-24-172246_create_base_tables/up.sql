CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    passhash VARCHAR(255) NOT NULL,
    created_utc INT NOT NULL,
    admin_level SMALLINT NOT NULL,
    is_activated BOOLEAN NOT NULL DEFAULT false,
    over_18 BOOLEAN NOT NULL DEFAULT false,
    creation_ip VARCHAR(15) NOT NULL DEFAULT '',
    bio VARCHAR(1000) DEFAULT '',
    bio_html VARCHAR(1000) DEFAULT '',
    referred_by INT DEFAULT NULL,
    is_banned BOOLEAN DEFAULT false,
    unban_utc INT DEFAULT 0,
    ban_reason VARCHAR(255) DEFAULT '',
    defaultsorting VARCHAR(10) DEFAULT 'hot',
    defaulttime VARCHAR(10) DEFAULT 'all',
    feed_nonce INT DEFAULT 0,
    login_nonce INT DEFAULT 0,
    has_profile BOOLEAN DEFAULT false,
    has_banner BOOLEAN DEFAULT false,
    reserved VARCHAR(256) DEFAULT NULL,
    is_nsfw BOOLEAN DEFAULT false,
    tos_agreed_utc INT DEFAULT 0,
    profile_nonce INT DEFAULT 0,
    banner_nonce INT DEFAULT 0,
    mfa_secret VARCHAR(16) DEFAULT NULL,
    hide_offensive BOOLEAN DEFAULT false,
    hide_bot BOOLEAN DEFAULT false,
    show_nsfl BOOLEAN DEFAULT false,
    is_private BOOLEAN DEFAULT false,
    is_deleted BOOLEAN DEFAULT false,
    delete_reason VARCHAR(500) DEFAULT '',
    filter_nsfw BOOLEAN DEFAULT false,
    stored_karma INT DEFAULT 0,
    stored_subscriber_count INT DEFAULT 0,
    auto_join_chat BOOLEAN DEFAULT false,
    is_nofollow BOOLEAN DEFAULT false,
    custom_filter_list VARCHAR(1000) DEFAULT '',
    discord_id VARCHAR(64) DEFAULT NULL,
    creation_region VARCHAR(2) DEFAULT NULL,
    ban_evade INT DEFAULT 0,
    profile_upload_ip VARCHAR(15) DEFAULT '',
    banner_upload_ip VARCHAR(15) DEFAULT '',
    profile_upload_region VARCHAR(2) DEFAULT '',
    banner_upload_region VARCHAR(2) DEFAULT '',
    color VARCHAR(6) DEFAULT '805ad5',
    secondary_color VARCHAR(6) DEFAULT 'ffff00',
    comment_signature VARCHAR(280) DEFAULT '',
    comment_signature_html VARCHAR(512) DEFAULT '',
    profile_set_utc INT DEFAULT 0,
    bannner_set_utc INT DEFAULT 0,
    original_username VARCHAR(255) DEFAULT '',
    name_changed_utc INT DEFAULT 0
);

CREATE TABLE mods(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    created_utc INTEGER DEFAULT 0,
    accepted BOOLEAN DEFAULT false,
    invite_rescinded BOOLEAN DEFAULT false,
    perm_content BOOLEAN DEFAULT false,
    perm_appearance BOOLEAN DEFAULT false,
    perm_config BOOLEAN DEFAULT false,
    perm_access BOOLEAN DEFAULT false,
    perm_full BOOLEAN DEFAULT false
);

CREATE TABLE bans(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    created_utc INTEGER DEFAULT 0,
    banning_mod_id INTEGER NOT NULL,
    is_active INTEGER NOT NULL,
    mod_note VARCHAR(128) DEFAULT ''
);

CREATE TABLE chatbans(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    created_utc INTEGER DEFAULT 0,
    banning_mod_id INTEGER NOT NULL
);

CREATE TABLE contributors(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    created_utc INTEGER NOT NULL,
    is_active BOOLEAN DEFAULT true,
    approving_mod_id INTEGER NOT NULL
);

CREATE TABLE postrels(
    id BIGSERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL  
);

CREATE TABLE boardblocks(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    created_utc INTEGER DEFAULT 0
);

CREATE TABLE categories(
    id SERIAL PRIMARY KEY,
    category_name VARCHAR(20) DEFAULT '',
    category_description VARCHAR(250) DEFAULT '',
    category_icon VARCHAR(256) DEFAULT '',
    category_color VARCHAR(128) default '805ad5',
    visible BOOLEAN DEFAULT true,
    is_nsfw BOOLEAN DEFAULT false
);

CREATE TABLE subcategories(
    id SERIAL PRIMARY KEY,
    cat_id INTEGER NOT NULL,
    subcat_name VARCHAR(20) DEFAULT '',
    subcat_description VARCHAR(250) DEFAULT '',
    _visible BOOLEAN DEFAULT true
);

CREATE TABLE boards(
    id SERIAL PRIMARY KEY,
    board_name VARCHAR(255) NOT NULL,
    created_utc INTEGER DEFAULT 0,
    board_description VARCHAR(512) DEFAULT NULL,
    board_description_html VARCHAR(1000) DEFAULT NULL,
    over_18 BOOLEAN DEFAULT false,
    is_nsfl BOOLEAN DEFAULT false,
    is_banned BOOLEAN DEFAULT false,
    has_banner BOOLEAN DEFAULT false,
    has_profile BOOLEAN DEFAULT false,
    creator_id INTEGER NOT NULL,
    ban_reason VARCHAR(256) DEFAULT NULL,
    color VARCHAR(8) DEFAULT '805ad5',
    restricted_posting BOOLEAN DEFAULT false,
    disallowbots BOOLEAN DEFAULT false,
    hide_banner_data BOOLEAN DEFAULT false,
    profile_nonce INTEGER DEFAULT 0,
    banner_nonce INTEGER DEFAULT 0,
    is_private BOOLEAN DEFAULT false,
    color_nonce INTEGER DEFAULT 0,
    rank_trending NUMERIC DEFAULT 0.0,
    stored_subscriber_count INTEGER DEFAULT 1,
    all_opt_out BOOLEAN DEFAULT false,
    is_locked_category BOOLEAN DEFAULT false,
    subcat_id INTEGER DEFAULT 0,
    secondary_color VARCHAR(6) DEFAULT 'ffffff',
    public_chat BOOLEAN DEFAULT false,
    motd VARCHAR(1000) DEFAULT '',
    css_nonce INTEGER DEFAULT 0,
    css VARCHAR(65536) DEFAULT ''     
);

CREATE TABLE badlinks(
    id SERIAL PRIMARY KEY,
    reason INTEGER NOT NULL,
    link VARCHAR(512) NOT NULL,
    autoban BOOLEAN DEFAULT false
);

CREATE TABLE domains(
    id SERIAL PRIMARY KEY,
    domain VARCHAR(512) NOT NULL,
    can_submit BOOLEAN DEFAULT true,
    can_comment BOOLEAN DEFAULT true,
    reason INTEGER DEFAULT 0,
    show_thumbnail BOOLEAN DEFAULT false,
    embed_function VARCHAR(64) DEFAULT NULL,
    embed_template VARCHAR(32) DEFAULT NULL
);

CREATE TABLE oauth_apps(
    id SERIAL PRIMARY KEY,
    client_id VARCHAR(64) NOT NULL,
    client_secret VARCHAR(128) NOT NULL,
    app_name VARCHAR(50) NOT NULL,
    redirect_uri VARCHAR(4096) NOT NULL,
    author_id INTEGER NOT NULL,
    is_banned BOOLEAN DEFAULT false,
    app_description VARCHAR(256) DEFAULT ''
);

CREATE TABLE client_auths(
    id SERIAL PRIMARY KEY,
    oauth_client INTEGER NOT NULL,
    oauth_code VARCHAR(128) DEFAULT '',
    user_id INTEGER NOT NULL,
    scope_identity BOOLEAN DEFAULT false,
    scope_create BOOLEAN DEFAULT false,
    scope_read BOOLEAN DEFAULT false,
    scope_update BOOLEAN DEFAULT false,
    scope_delete BOOLEAN DEFAULT false,
    scope_vote BOOLEAN DEFAULT false,
    scope_moderator BOOLEAN DEFAULT false,
    access_token VARCHAR(128) DEFAULT '',
    refresh_token VARCHAR(128) DEFAULT '',
    access_token_expire_utc INTEGER DEFAULT 0
);

CREATE TABLE submissions(
    id BIGSERIAL PRIMARY KEY,
    author_id INTEGER NOT NULL,
    repost_id INTEGER DEFAULT 0,
    edited_utc INTEGER DEFAULT 0,
    created_utc INTEGER DEFAULT 0,
    is_banned BOOLEAN DEFAULT false,
    deleted_utc INTEGER DEFAULT 0,
    purged_utc INTEGER DEFAULT 0,
    distinguish_level SMALLINT DEFAULT 0,
    gm_distinguish SMALLINT DEFAULT 0,
    created_str VARCHAR(255) DEFAULT NULL,
    stickied BOOLEAN DEFAULT false,
    domain_ref INTEGER DEFAULT 0
);

CREATE TABLE badge_defs(
    id SERIAL PRIMARY KEY,
    badge_name VARCHAR(64) DEFAULT '',
    badge_description VARCHAR(64) DEFAULT '',
    badge_icon VARCHAR(64) DEFAULT '',
    badge_kind SMALLINT DEFAULT 1,
    badge_rank SMALLINT DEFAULT 1,
    qualification_expr VARCHAR(128) DEFAULT NULL
);

CREATE TABLE badges(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    badge_id INTEGER NOT NULL,
    badge_description VARCHAR(64) DEFAULT '',
    badge_url VARCHAR(256) DEFAULT '',
    created_utc INTEGER DEFAULT 0
);

CREATE TABLE alts(
    id SERIAL PRIMARY KEY,
    user1 INTEGER NOT NULL,
    user2 INTEGER NOT NULL,
    is_manual BOOLEAN DEFAULT false
);

CREATE TABLE badwords(
    id SERIAL PRIMARY KEY,
    keyword VARCHAR(64) DEFAULT '',
    regex VARCHAR(256) DEFAULT ''
);