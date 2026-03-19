-- Core tables: site, users, boards, board_moderators

-- ============================================================
-- site — single-row instance configuration
-- ============================================================

CREATE TABLE site (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(50) NOT NULL,
    description     VARCHAR(255),
    icon            VARCHAR(255),
    homepage_banner TEXT,
    primary_color   VARCHAR(25) NOT NULL DEFAULT '#000000',
    secondary_color VARCHAR(25) NOT NULL DEFAULT '#FFFFFF',
    hover_color     VARCHAR(25) NOT NULL DEFAULT '#333333',
    welcome_message VARCHAR(255),
    legal_information TEXT,
    default_theme   TEXT NOT NULL DEFAULT 'default',
    default_post_listing_type listing_type NOT NULL DEFAULT 'all',
    default_avatar  TEXT,

    -- Registration
    registration_mode           registration_mode NOT NULL DEFAULT 'open',
    application_question        TEXT,
    is_site_setup               BOOLEAN NOT NULL DEFAULT false,
    is_private                  BOOLEAN NOT NULL DEFAULT false,
    require_email_verification  BOOLEAN NOT NULL DEFAULT false,
    application_email_admins    BOOLEAN NOT NULL DEFAULT false,
    captcha_enabled             BOOLEAN NOT NULL DEFAULT false,
    captcha_difficulty          VARCHAR(255) NOT NULL DEFAULT 'medium',

    -- Content policy
    enable_downvotes    BOOLEAN NOT NULL DEFAULT true,
    enable_nsfw         BOOLEAN NOT NULL DEFAULT false,
    enable_nsfw_tagging BOOLEAN NOT NULL DEFAULT false,
    hide_modlog_mod_names BOOLEAN NOT NULL DEFAULT false,
    reports_email_admins  BOOLEAN NOT NULL DEFAULT false,

    -- Boards
    boards_enabled              BOOLEAN NOT NULL DEFAULT true,
    board_creation_admin_only   BOOLEAN NOT NULL DEFAULT false,
    board_creation_mode         VARCHAR(20) NOT NULL DEFAULT 'open',

    -- Trust system
    trusted_user_min_reputation       INT NOT NULL DEFAULT 0,
    trusted_user_min_account_age_days INT NOT NULL DEFAULT 0,
    trusted_user_manual_approval      BOOLEAN NOT NULL DEFAULT false,
    trusted_user_min_posts            INT NOT NULL DEFAULT 0,

    -- Content filtering
    allowed_post_types              TEXT,
    word_filter_enabled             BOOLEAN NOT NULL DEFAULT false,
    filtered_words                  TEXT,
    word_filter_applies_to_posts    BOOLEAN NOT NULL DEFAULT true,
    word_filter_applies_to_comments BOOLEAN NOT NULL DEFAULT true,
    word_filter_applies_to_usernames BOOLEAN NOT NULL DEFAULT false,
    link_filter_enabled             BOOLEAN NOT NULL DEFAULT false,
    banned_domains                  TEXT,
    approved_image_hosts            TEXT,
    image_embed_hosts_only          BOOLEAN NOT NULL DEFAULT false,

    -- Emoji
    emoji_enabled           BOOLEAN NOT NULL DEFAULT true,
    max_emojis_per_post     INT,
    max_emojis_per_comment  INT,
    emoji_max_file_size_mb  INT NOT NULL DEFAULT 1,
    board_emojis_enabled    BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT add_updated_at_trigger('site');

-- ============================================================
-- users
-- ============================================================

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(30) NOT NULL,
    display_name    VARCHAR(30),
    email           TEXT,
    passhash        TEXT NOT NULL,

    -- Flags
    is_email_verified   BOOLEAN NOT NULL DEFAULT false,
    is_banned           BOOLEAN NOT NULL DEFAULT false,
    is_admin            BOOLEAN NOT NULL DEFAULT false,
    admin_level         INT NOT NULL DEFAULT 0,
    is_bot_account      BOOLEAN NOT NULL DEFAULT false,
    is_board_creation_approved BOOLEAN NOT NULL DEFAULT false,
    is_application_accepted    BOOLEAN NOT NULL DEFAULT false,

    unban_date      TIMESTAMPTZ,

    -- Profile
    bio                 TEXT,
    bio_html            TEXT,
    signature           TEXT,
    avatar              TEXT,
    banner              TEXT,
    profile_background  TEXT,
    avatar_frame        TEXT,
    profile_music       TEXT,
    profile_music_youtube TEXT,

    -- Preferences
    show_nsfw               BOOLEAN NOT NULL DEFAULT false,
    show_bots               BOOLEAN NOT NULL DEFAULT true,
    theme                   TEXT NOT NULL DEFAULT 'default',
    default_sort_type       sort_type NOT NULL DEFAULT 'hot',
    default_listing_type    listing_type NOT NULL DEFAULT 'all',
    interface_language      TEXT NOT NULL DEFAULT 'en',
    is_email_notifications_enabled BOOLEAN NOT NULL DEFAULT false,
    editor_mode             editor_mode NOT NULL DEFAULT 'richtext',

    last_seen_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT users_name_unique UNIQUE (name)
);

SELECT add_updated_at_trigger('users');

CREATE INDEX idx_users_email ON users (email) WHERE email IS NOT NULL;
CREATE INDEX idx_users_is_banned ON users (is_banned) WHERE is_banned = true;
CREATE INDEX idx_users_created_at ON users (created_at);

-- ============================================================
-- boards
-- ============================================================

CREATE TABLE boards (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(50) NOT NULL,
    title           VARCHAR(150) NOT NULL,
    description     TEXT,
    sidebar         VARCHAR(10000),
    sidebar_html    TEXT,
    icon            TEXT,
    banner          TEXT,
    primary_color   VARCHAR(25) NOT NULL DEFAULT '#000000',
    secondary_color VARCHAR(25) NOT NULL DEFAULT '#FFFFFF',
    hover_color     VARCHAR(25) NOT NULL DEFAULT '#333333',

    -- Flags
    is_nsfw                     BOOLEAN NOT NULL DEFAULT false,
    is_hidden                   BOOLEAN NOT NULL DEFAULT false,
    is_removed                  BOOLEAN NOT NULL DEFAULT false,
    is_banned                   BOOLEAN NOT NULL DEFAULT false,
    is_posting_restricted_to_mods BOOLEAN NOT NULL DEFAULT false,
    exclude_from_all            BOOLEAN NOT NULL DEFAULT false,

    -- Ban info
    ban_reason          VARCHAR(512),
    public_ban_reason   TEXT,
    banned_by           UUID REFERENCES users(id) ON DELETE SET NULL,
    banned_at           TIMESTAMPTZ,

    -- Sections
    section_config      INT NOT NULL DEFAULT 0,
    section_order       TEXT,
    default_section     TEXT,

    -- Wiki
    wiki_enabled                    BOOLEAN NOT NULL DEFAULT false,
    wiki_require_approval           BOOLEAN,
    wiki_default_view_permission    wiki_permission NOT NULL DEFAULT 'public',
    wiki_default_edit_permission    wiki_permission NOT NULL DEFAULT 'members',

    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ,

    CONSTRAINT boards_name_unique UNIQUE (name)
);

SELECT add_updated_at_trigger('boards');

CREATE INDEX idx_boards_created_at ON boards (created_at);

-- ============================================================
-- board_moderators
-- ============================================================

CREATE TABLE board_moderators (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id            UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    permissions         INT NOT NULL DEFAULT 0,
    rank                INT NOT NULL DEFAULT 0,
    is_invite_accepted  BOOLEAN NOT NULL DEFAULT false,
    invite_accepted_at  TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT board_moderators_unique UNIQUE (board_id, user_id)
);

CREATE INDEX idx_board_moderators_user ON board_moderators (user_id);

-- Auto-assign rank on insert
CREATE OR REPLACE FUNCTION board_moderators_set_rank()
RETURNS TRIGGER AS $$
BEGIN
    NEW.rank := COALESCE(
        (SELECT MAX(rank) + 1 FROM board_moderators WHERE board_id = NEW.board_id),
        0
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER board_moderators_insert_rank
    BEFORE INSERT ON board_moderators
    FOR EACH ROW EXECUTE FUNCTION board_moderators_set_rank();

-- Re-rank on delete
CREATE OR REPLACE FUNCTION board_moderators_rerank()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE board_moderators
    SET rank = rank - 1
    WHERE board_id = OLD.board_id AND rank > OLD.rank;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER board_moderators_delete_rerank
    AFTER DELETE ON board_moderators
    FOR EACH ROW EXECUTE FUNCTION board_moderators_rerank();

-- Set invite_accepted_at when invite is accepted
CREATE OR REPLACE FUNCTION board_moderators_accept_invite()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_invite_accepted = true AND OLD.is_invite_accepted = false THEN
        NEW.invite_accepted_at = now();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER board_moderators_invite_accepted
    BEFORE UPDATE ON board_moderators
    FOR EACH ROW EXECUTE FUNCTION board_moderators_accept_invite();
