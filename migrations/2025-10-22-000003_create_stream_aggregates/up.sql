-- Migration 3: Stream Aggregates and Trigger Functions
-- This migration creates aggregates for stream statistics and automatic update triggers

-- ==================== STREAM AGGREGATES TABLE ====================
-- Pre-computed statistics for streams to optimize query performance
CREATE TABLE stream_aggregates (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL UNIQUE,

    -- Subscription counts
    flair_subscription_count INTEGER NOT NULL DEFAULT 0,
    board_subscription_count INTEGER NOT NULL DEFAULT 0,
    total_subscription_count INTEGER NOT NULL DEFAULT 0, -- flair + board counts

    -- Follower count (excluding creator)
    follower_count INTEGER NOT NULL DEFAULT 0,

    -- Activity metrics (updated periodically, not via triggers)
    posts_last_day INTEGER NOT NULL DEFAULT 0,
    posts_last_week INTEGER NOT NULL DEFAULT 0,
    posts_last_month INTEGER NOT NULL DEFAULT 0,

    -- Metadata
    creation_date TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP,

    -- Foreign key
    CONSTRAINT fk_stream_agg_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE
);

-- Index for efficient lookups
CREATE INDEX idx_stream_agg_stream_id ON stream_aggregates(stream_id);
CREATE INDEX idx_stream_agg_follower_count ON stream_aggregates(follower_count DESC);
CREATE INDEX idx_stream_agg_total_subs ON stream_aggregates(total_subscription_count DESC);

COMMENT ON TABLE stream_aggregates IS 'Pre-computed statistics for streams to optimize query performance';
COMMENT ON COLUMN stream_aggregates.total_subscription_count IS 'Sum of flair_subscription_count and board_subscription_count';

-- ==================== TRIGGER FUNCTIONS ====================

-- Trigger function: Create aggregate when stream is created
CREATE OR REPLACE FUNCTION stream_aggregates_stream()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO stream_aggregates (stream_id) VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM stream_aggregates WHERE stream_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger function: Update flair subscription count
CREATE OR REPLACE FUNCTION stream_aggregates_flair_subscription()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE stream_aggregates
        SET
            flair_subscription_count = flair_subscription_count + 1,
            total_subscription_count = total_subscription_count + 1,
            updated_at = NOW()
        WHERE stream_id = NEW.stream_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE stream_aggregates
        SET
            flair_subscription_count = GREATEST(0, flair_subscription_count - 1),
            total_subscription_count = GREATEST(0, total_subscription_count - 1),
            updated_at = NOW()
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger function: Update board subscription count
CREATE OR REPLACE FUNCTION stream_aggregates_board_subscription()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE stream_aggregates
        SET
            board_subscription_count = board_subscription_count + 1,
            total_subscription_count = total_subscription_count + 1,
            updated_at = NOW()
        WHERE stream_id = NEW.stream_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE stream_aggregates
        SET
            board_subscription_count = GREATEST(0, board_subscription_count - 1),
            total_subscription_count = GREATEST(0, total_subscription_count - 1),
            updated_at = NOW()
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger function: Update follower count
CREATE OR REPLACE FUNCTION stream_aggregates_follower()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE stream_aggregates
        SET
            follower_count = follower_count + 1,
            updated_at = NOW()
        WHERE stream_id = NEW.stream_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE stream_aggregates
        SET
            follower_count = GREATEST(0, follower_count - 1),
            updated_at = NOW()
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- ==================== ATTACH TRIGGERS ====================

-- Trigger on streams table
CREATE TRIGGER stream_aggregates_stream_trigger
    AFTER INSERT OR DELETE ON streams
    FOR EACH ROW
    EXECUTE FUNCTION stream_aggregates_stream();

-- Trigger on stream_flair_subscriptions table
CREATE TRIGGER stream_aggregates_flair_subscription_trigger
    AFTER INSERT OR DELETE ON stream_flair_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION stream_aggregates_flair_subscription();

-- Trigger on stream_board_subscriptions table
CREATE TRIGGER stream_aggregates_board_subscription_trigger
    AFTER INSERT OR DELETE ON stream_board_subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION stream_aggregates_board_subscription();

-- Trigger on stream_followers table
CREATE TRIGGER stream_aggregates_follower_trigger
    AFTER INSERT OR DELETE ON stream_followers
    FOR EACH ROW
    EXECUTE FUNCTION stream_aggregates_follower();

-- ==================== HELPER FUNCTION ====================

-- Function to recalculate all stream aggregates (for data repairs or updates)
CREATE OR REPLACE FUNCTION recalculate_stream_aggregates(p_stream_id INTEGER DEFAULT NULL)
RETURNS VOID AS $$
BEGIN
    IF p_stream_id IS NOT NULL THEN
        -- Recalculate for specific stream
        UPDATE stream_aggregates sa
        SET
            flair_subscription_count = (
                SELECT COUNT(*) FROM stream_flair_subscriptions
                WHERE stream_id = p_stream_id
            ),
            board_subscription_count = (
                SELECT COUNT(*) FROM stream_board_subscriptions
                WHERE stream_id = p_stream_id
            ),
            total_subscription_count = (
                SELECT COUNT(*) FROM stream_flair_subscriptions
                WHERE stream_id = p_stream_id
            ) + (
                SELECT COUNT(*) FROM stream_board_subscriptions
                WHERE stream_id = p_stream_id
            ),
            follower_count = (
                SELECT COUNT(*) FROM stream_followers
                WHERE stream_id = p_stream_id
            ),
            updated_at = NOW()
        WHERE sa.stream_id = p_stream_id;
    ELSE
        -- Recalculate for all streams
        UPDATE stream_aggregates sa
        SET
            flair_subscription_count = COALESCE((
                SELECT COUNT(*) FROM stream_flair_subscriptions sfs
                WHERE sfs.stream_id = sa.stream_id
            ), 0),
            board_subscription_count = COALESCE((
                SELECT COUNT(*) FROM stream_board_subscriptions sbs
                WHERE sbs.stream_id = sa.stream_id
            ), 0),
            total_subscription_count = COALESCE((
                SELECT COUNT(*) FROM stream_flair_subscriptions sfs
                WHERE sfs.stream_id = sa.stream_id
            ), 0) + COALESCE((
                SELECT COUNT(*) FROM stream_board_subscriptions sbs
                WHERE sbs.stream_id = sa.stream_id
            ), 0),
            follower_count = COALESCE((
                SELECT COUNT(*) FROM stream_followers sf
                WHERE sf.stream_id = sa.stream_id
            ), 0),
            updated_at = NOW();
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION recalculate_stream_aggregates IS 'Recalculate stream aggregates for a specific stream or all streams. Use for data repairs.';
