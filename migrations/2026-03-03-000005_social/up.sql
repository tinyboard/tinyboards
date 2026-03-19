-- Social tables: subscriptions, bans, blocks, saves, follows

-- ============================================================
-- board_subscribers
-- ============================================================

CREATE TABLE board_subscribers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_pending  BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT board_subscribers_unique UNIQUE (board_id, user_id)
);

CREATE INDEX idx_board_subscribers_user ON board_subscribers (user_id);

-- Trigger: update subscriber count in board_aggregates
CREATE OR REPLACE FUNCTION trg_board_aggregates_subscriber_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE board_aggregates SET subscribers = subscribers + 1 WHERE board_id = NEW.board_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE board_aggregates SET subscribers = subscribers - 1 WHERE board_id = OLD.board_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER board_aggregates_subscriber_count
    AFTER INSERT OR DELETE ON board_subscribers
    FOR EACH ROW EXECUTE FUNCTION trg_board_aggregates_subscriber_count();

-- ============================================================
-- board_user_bans
-- ============================================================

CREATE TABLE board_user_bans (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT board_user_bans_unique UNIQUE (board_id, user_id)
);

-- ============================================================
-- user_bans (site-level)
-- ============================================================

CREATE TABLE user_bans (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    reason      TEXT,
    expires_at  TIMESTAMPTZ,
    banned_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_bans_user ON user_bans (user_id);

-- ============================================================
-- user_blocks
-- ============================================================

CREATE TABLE user_blocks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT user_blocks_unique UNIQUE (user_id, target_id)
);

-- ============================================================
-- board_blocks
-- ============================================================

CREATE TABLE board_blocks (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT board_blocks_unique UNIQUE (user_id, board_id)
);

-- ============================================================
-- post_saved
-- ============================================================

CREATE TABLE post_saved (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id     UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT post_saved_unique UNIQUE (post_id, user_id)
);

-- ============================================================
-- comment_saved
-- ============================================================

CREATE TABLE comment_saved (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id  UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT comment_saved_unique UNIQUE (comment_id, user_id)
);

-- ============================================================
-- post_hidden
-- ============================================================

CREATE TABLE post_hidden (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id     UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT post_hidden_unique UNIQUE (post_id, user_id)
);

CREATE INDEX idx_post_hidden_user ON post_hidden (user_id);

-- ============================================================
-- user_follows
-- ============================================================

CREATE TABLE user_follows (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    follower_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_pending      BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT user_follows_unique UNIQUE (user_id, follower_id)
);

CREATE INDEX idx_user_follows_follower ON user_follows (follower_id);

-- ============================================================
-- Language junction tables
-- ============================================================

CREATE TABLE board_languages (
    id          SERIAL PRIMARY KEY,
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    language_id INT NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    CONSTRAINT board_languages_unique UNIQUE (board_id, language_id)
);

CREATE TABLE site_languages (
    id          SERIAL PRIMARY KEY,
    site_id     UUID NOT NULL REFERENCES site(id) ON DELETE CASCADE,
    language_id INT NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    CONSTRAINT site_languages_unique UNIQUE (site_id, language_id)
);

CREATE TABLE user_languages (
    id          SERIAL PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    language_id INT NOT NULL REFERENCES languages(id) ON DELETE CASCADE,
    CONSTRAINT user_languages_unique UNIQUE (user_id, language_id)
);
