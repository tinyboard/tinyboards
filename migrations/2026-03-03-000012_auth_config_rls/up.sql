-- Auth, config, and row-level security

-- ============================================================
-- secrets (single-row JWT secret store)
-- ============================================================

CREATE TABLE secrets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jwt_secret  VARCHAR NOT NULL
);

-- ============================================================
-- password_resets
-- ============================================================

CREATE TABLE password_resets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reset_token     TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_password_resets_user ON password_resets (user_id);
CREATE INDEX idx_password_resets_token ON password_resets (reset_token);

-- ============================================================
-- email_verification
-- ============================================================

CREATE TABLE email_verification (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email               TEXT NOT NULL,
    verification_code   TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_email_verification_user ON email_verification (user_id);

-- ============================================================
-- registration_applications
-- ============================================================

CREATE TABLE registration_applications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    answer      TEXT NOT NULL,
    admin_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    deny_reason TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_registration_applications_user ON registration_applications (user_id);

-- ============================================================
-- site_invites
-- ============================================================

CREATE TABLE site_invites (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    verification_code   TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- rate_limits
-- ============================================================

CREATE TABLE rate_limits (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id             UUID NOT NULL REFERENCES site(id) ON DELETE CASCADE,
    message             INT NOT NULL DEFAULT 180,
    message_per_second  INT NOT NULL DEFAULT 60,
    post                INT NOT NULL DEFAULT 6,
    post_per_second     INT NOT NULL DEFAULT 600,
    register            INT NOT NULL DEFAULT 3,
    register_per_second INT NOT NULL DEFAULT 3600,
    image               INT NOT NULL DEFAULT 6,
    image_per_second    INT NOT NULL DEFAULT 3600,
    comment             INT NOT NULL DEFAULT 6,
    comment_per_second  INT NOT NULL DEFAULT 600,
    search              INT NOT NULL DEFAULT 60,
    search_per_second   INT NOT NULL DEFAULT 600,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT rate_limits_site_unique UNIQUE (site_id)
);

SELECT add_updated_at_trigger('rate_limits');

-- ============================================================
-- Row-Level Security
-- ============================================================

-- Enable RLS on tables containing user-specific data
ALTER TABLE private_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE post_saved ENABLE ROW LEVEL SECURITY;
ALTER TABLE comment_saved ENABLE ROW LEVEL SECURITY;
ALTER TABLE post_hidden ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_blocks ENABLE ROW LEVEL SECURITY;

-- Private messages: users see only their own sent/received
CREATE POLICY pm_owner_policy ON private_messages
    USING (creator_id = current_app_user_id() OR recipient_id = current_app_user_id());

-- Notifications: users see only their own
CREATE POLICY notification_owner_policy ON notifications
    USING (recipient_user_id = current_app_user_id());

-- Notification settings: users see only their own
CREATE POLICY notification_settings_owner_policy ON notification_settings
    USING (user_id = current_app_user_id());

-- Saved posts: users see only their own
CREATE POLICY post_saved_owner_policy ON post_saved
    USING (user_id = current_app_user_id());

-- Saved comments: users see only their own
CREATE POLICY comment_saved_owner_policy ON comment_saved
    USING (user_id = current_app_user_id());

-- Hidden posts: users see only their own
CREATE POLICY post_hidden_owner_policy ON post_hidden
    USING (user_id = current_app_user_id());

-- User blocks: users see only their own
CREATE POLICY user_blocks_owner_policy ON user_blocks
    USING (user_id = current_app_user_id());
