-- Migration 2: Stream Subscriptions (Flair and Board-based)
-- This migration creates tables for both flair-based and board-based stream subscriptions
-- allowing streams to subscribe to specific flairs OR entire boards

-- ==================== STREAM FLAIR SUBSCRIPTIONS ====================
-- Subscribe to specific flairs from specific boards
-- This allows granular control: "I want posts with 'News' flair from +technology and 'Breaking' flair from +worldnews"
CREATE TABLE stream_flair_subscriptions (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    flair_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_stream_flair_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_stream_flair_board
        FOREIGN KEY (board_id)
        REFERENCES boards(id)
        ON DELETE CASCADE
    -- Note: flair_id will reference post_flairs table when it's created
    -- For now, we store it as integer to avoid circular dependency
);

-- Prevent duplicate flair subscriptions in the same stream
CREATE UNIQUE INDEX idx_stream_flair_unique
    ON stream_flair_subscriptions(stream_id, board_id, flair_id);

-- Indexes for efficient querying
CREATE INDEX idx_stream_flair_stream_id ON stream_flair_subscriptions(stream_id);
CREATE INDEX idx_stream_flair_board_id ON stream_flair_subscriptions(board_id);
CREATE INDEX idx_stream_flair_flair_id ON stream_flair_subscriptions(flair_id);

COMMENT ON TABLE stream_flair_subscriptions IS 'Flair-based subscriptions: stream subscribes to specific flairs from specific boards';
COMMENT ON COLUMN stream_flair_subscriptions.flair_id IS 'References post_flairs.id - will be enforced when flair system is implemented';

-- ==================== STREAM BOARD SUBSCRIPTIONS ====================
-- Subscribe to ALL content from specific boards (ignoring flairs)
-- This allows broad subscriptions: "I want ALL posts from +technology board, regardless of flair"
CREATE TABLE stream_board_subscriptions (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    include_all_posts BOOLEAN NOT NULL DEFAULT true, -- Future flexibility for filtering
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_stream_board_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_stream_board_board
        FOREIGN KEY (board_id)
        REFERENCES boards(id)
        ON DELETE CASCADE
);

-- Prevent duplicate board subscriptions in the same stream
CREATE UNIQUE INDEX idx_stream_board_unique
    ON stream_board_subscriptions(stream_id, board_id);

-- Indexes for efficient querying
CREATE INDEX idx_stream_board_stream_id ON stream_board_subscriptions(stream_id);
CREATE INDEX idx_stream_board_board_id ON stream_board_subscriptions(board_id);

COMMENT ON TABLE stream_board_subscriptions IS 'Board-based subscriptions: stream subscribes to ALL posts from specific boards';
COMMENT ON COLUMN stream_board_subscriptions.include_all_posts IS 'Reserved for future filtering options (e.g., only featured posts, only top posts)';

-- ==================== STREAM FOLLOWERS ====================
-- Users can follow streams created by others (for public/shared streams)
CREATE TABLE stream_followers (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    followed_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Navbar integration
    added_to_navbar BOOLEAN NOT NULL DEFAULT false,
    navbar_position INTEGER, -- Optional position in navbar (1 = first, 2 = second, etc.)

    -- Foreign keys
    CONSTRAINT fk_stream_follower_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_stream_follower_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Prevent duplicate follows
CREATE UNIQUE INDEX idx_stream_follower_unique
    ON stream_followers(stream_id, user_id);

-- Indexes for efficient querying
CREATE INDEX idx_stream_follower_stream_id ON stream_followers(stream_id);
CREATE INDEX idx_stream_follower_user_id ON stream_followers(user_id);
CREATE INDEX idx_stream_follower_navbar ON stream_followers(user_id, added_to_navbar)
    WHERE added_to_navbar = true;

COMMENT ON TABLE stream_followers IS 'Tracks which users follow which streams (for public/shared streams)';
COMMENT ON COLUMN stream_followers.navbar_position IS 'Position in user navbar. NULL means not explicitly ordered.';
