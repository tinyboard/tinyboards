-- Messaging and notification tables

-- ============================================================
-- private_messages (consolidated from old messages + private_message)
-- ============================================================

CREATE TABLE private_messages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_id    UUID REFERENCES users(id) ON DELETE CASCADE,
    recipient_board_id UUID REFERENCES boards(id) ON DELETE CASCADE,
    subject         VARCHAR(200),
    body            TEXT NOT NULL,
    body_html       TEXT NOT NULL,
    is_read         BOOLEAN NOT NULL DEFAULT false,
    is_sender_hidden BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    -- At least one recipient must be set
    CONSTRAINT pm_has_recipient CHECK (
        recipient_id IS NOT NULL OR recipient_board_id IS NOT NULL
    )
);

SELECT add_updated_at_trigger('private_messages');

CREATE INDEX idx_private_messages_recipient ON private_messages (recipient_id, is_read)
    WHERE recipient_id IS NOT NULL;
CREATE INDEX idx_private_messages_creator ON private_messages (creator_id);
CREATE INDEX idx_private_messages_board ON private_messages (recipient_board_id)
    WHERE recipient_board_id IS NOT NULL;

-- ============================================================
-- notifications (partitioned by created_at)
-- ============================================================

CREATE TABLE notifications (
    id                  UUID NOT NULL DEFAULT gen_random_uuid(),
    kind                notification_kind NOT NULL,
    recipient_user_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    comment_id          UUID REFERENCES comments(id) ON DELETE CASCADE,
    post_id             UUID REFERENCES posts(id) ON DELETE CASCADE,
    message_id          UUID REFERENCES private_messages(id) ON DELETE CASCADE,
    is_read             BOOLEAN NOT NULL DEFAULT false,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

CREATE TABLE notifications_default PARTITION OF notifications DEFAULT;

CREATE TABLE notifications_2026_01 PARTITION OF notifications
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE notifications_2026_02 PARTITION OF notifications
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
CREATE TABLE notifications_2026_03 PARTITION OF notifications
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');
CREATE TABLE notifications_2026_04 PARTITION OF notifications
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');
CREATE TABLE notifications_2026_05 PARTITION OF notifications
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');
CREATE TABLE notifications_2026_06 PARTITION OF notifications
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

CREATE INDEX idx_notifications_recipient_unread
    ON notifications (recipient_user_id, is_read) WHERE is_read = false;
CREATE INDEX idx_notifications_recipient_created
    ON notifications (recipient_user_id, created_at DESC);

-- ============================================================
-- notification_settings
-- ============================================================

CREATE TABLE notification_settings (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_email_enabled            BOOLEAN NOT NULL DEFAULT false,
    is_comment_replies_enabled  BOOLEAN NOT NULL DEFAULT true,
    is_post_replies_enabled     BOOLEAN NOT NULL DEFAULT true,
    is_mentions_enabled         BOOLEAN NOT NULL DEFAULT true,
    is_private_messages_enabled BOOLEAN NOT NULL DEFAULT true,
    is_board_invites_enabled    BOOLEAN NOT NULL DEFAULT true,
    is_moderator_actions_enabled BOOLEAN NOT NULL DEFAULT true,
    is_system_notifications_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT notification_settings_user_unique UNIQUE (user_id)
);

SELECT add_updated_at_trigger('notification_settings');
