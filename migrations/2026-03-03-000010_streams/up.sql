-- Stream (curated feed) tables

-- ============================================================
-- streams
-- ============================================================

CREATE TABLE streams (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    slug            VARCHAR(100) NOT NULL,
    description     TEXT,
    icon            VARCHAR(255),
    color           VARCHAR(25),
    is_public       BOOLEAN NOT NULL DEFAULT false,
    is_discoverable BOOLEAN NOT NULL DEFAULT false,
    share_token     VARCHAR(64),
    sort_type       sort_type NOT NULL DEFAULT 'hot',
    time_range      VARCHAR(20),
    show_nsfw       BOOLEAN NOT NULL DEFAULT false,
    max_posts_per_board INT,
    last_viewed_at  TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT streams_creator_slug_unique UNIQUE (creator_id, slug)
);

SELECT add_updated_at_trigger('streams');

CREATE INDEX idx_streams_creator ON streams (creator_id);
CREATE INDEX idx_streams_public ON streams (is_public) WHERE is_public = true;
CREATE INDEX idx_streams_discoverable ON streams (is_discoverable) WHERE is_discoverable = true;
CREATE INDEX idx_streams_share_token ON streams (share_token) WHERE share_token IS NOT NULL;
CREATE INDEX idx_streams_slug ON streams (slug);

-- ============================================================
-- stream_board_subscriptions
-- ============================================================

CREATE TABLE stream_board_subscriptions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id           UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    board_id            UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    include_all_posts   BOOLEAN NOT NULL DEFAULT true,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_board_sub_unique UNIQUE (stream_id, board_id)
);

-- ============================================================
-- stream_flair_subscriptions
-- ============================================================

CREATE TABLE stream_flair_subscriptions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    flair_id    INT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_flair_sub_unique UNIQUE (stream_id, board_id, flair_id)
);

-- ============================================================
-- stream_followers
-- ============================================================

CREATE TABLE stream_followers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id       UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_to_navbar BOOLEAN NOT NULL DEFAULT false,
    navbar_position INT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_followers_unique UNIQUE (stream_id, user_id)
);

-- ============================================================
-- stream_excluded_boards
-- ============================================================

CREATE TABLE stream_excluded_boards (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_excluded_boards_unique UNIQUE (stream_id, board_id)
);

-- ============================================================
-- stream_excluded_users
-- ============================================================

CREATE TABLE stream_excluded_users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_excluded_users_unique UNIQUE (stream_id, user_id)
);

-- ============================================================
-- stream_tags
-- ============================================================

CREATE TABLE stream_tags (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    tag         VARCHAR(50) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_tags_unique UNIQUE (stream_id, tag)
);

-- ============================================================
-- stream_view_history
-- ============================================================

CREATE TABLE stream_view_history (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id   UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    viewed_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_stream_view_history_stream ON stream_view_history (stream_id);

-- ============================================================
-- stream_aggregates
-- ============================================================

CREATE TABLE stream_aggregates (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id                   UUID NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    flair_subscription_count    INT NOT NULL DEFAULT 0,
    board_subscription_count    INT NOT NULL DEFAULT 0,
    total_subscription_count    INT NOT NULL DEFAULT 0,
    follower_count              INT NOT NULL DEFAULT 0,
    posts_last_day              INT NOT NULL DEFAULT 0,
    posts_last_week             INT NOT NULL DEFAULT 0,
    posts_last_month            INT NOT NULL DEFAULT 0,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT stream_aggregates_stream_unique UNIQUE (stream_id)
);

SELECT add_updated_at_trigger('stream_aggregates');

-- ============================================================
-- Stream triggers
-- ============================================================

-- Auto-create stream_aggregates on stream insert
CREATE OR REPLACE FUNCTION trg_stream_aggregates_on_stream()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO stream_aggregates (stream_id) VALUES (NEW.id);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER stream_aggregates_on_stream
    AFTER INSERT ON streams
    FOR EACH ROW EXECUTE FUNCTION trg_stream_aggregates_on_stream();

-- Update counts for board subscriptions
CREATE OR REPLACE FUNCTION trg_stream_board_sub_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE stream_aggregates
        SET board_subscription_count = board_subscription_count + 1,
            total_subscription_count = total_subscription_count + 1
        WHERE stream_id = NEW.stream_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE stream_aggregates
        SET board_subscription_count = board_subscription_count - 1,
            total_subscription_count = total_subscription_count - 1
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER stream_board_sub_count
    AFTER INSERT OR DELETE ON stream_board_subscriptions
    FOR EACH ROW EXECUTE FUNCTION trg_stream_board_sub_count();

-- Update counts for flair subscriptions
CREATE OR REPLACE FUNCTION trg_stream_flair_sub_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE stream_aggregates
        SET flair_subscription_count = flair_subscription_count + 1,
            total_subscription_count = total_subscription_count + 1
        WHERE stream_id = NEW.stream_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE stream_aggregates
        SET flair_subscription_count = flair_subscription_count - 1,
            total_subscription_count = total_subscription_count - 1
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER stream_flair_sub_count
    AFTER INSERT OR DELETE ON stream_flair_subscriptions
    FOR EACH ROW EXECUTE FUNCTION trg_stream_flair_sub_count();

-- Update follower count
CREATE OR REPLACE FUNCTION trg_stream_follower_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE stream_aggregates SET follower_count = follower_count + 1
        WHERE stream_id = NEW.stream_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE stream_aggregates SET follower_count = follower_count - 1
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER stream_follower_count
    AFTER INSERT OR DELETE ON stream_followers
    FOR EACH ROW EXECUTE FUNCTION trg_stream_follower_count();
