-- Migration 4: Stream Settings and Additional Configuration
-- This migration adds optional settings and features for enhanced stream functionality

-- ==================== STREAM VIEW HISTORY ====================
-- Track when streams are viewed for analytics and recommendations
CREATE TABLE stream_view_history (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    user_id INTEGER, -- NULL for anonymous views
    viewed_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_stream_view_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_stream_view_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE SET NULL
);

-- Indexes for analytics queries
CREATE INDEX idx_stream_view_stream_id ON stream_view_history(stream_id, viewed_at DESC);
CREATE INDEX idx_stream_view_user_id ON stream_view_history(user_id, viewed_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_stream_view_recent ON stream_view_history(viewed_at DESC);

COMMENT ON TABLE stream_view_history IS 'Tracks stream views for analytics and trending calculations';

-- ==================== STREAM TAGS ====================
-- Optional tagging system for stream discovery and categorization
CREATE TABLE stream_tags (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    tag VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign key
    CONSTRAINT fk_stream_tag_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE
);

-- Prevent duplicate tags on same stream
CREATE UNIQUE INDEX idx_stream_tag_unique ON stream_tags(stream_id, tag);

-- Indexes for tag search and filtering
CREATE INDEX idx_stream_tag_stream_id ON stream_tags(stream_id);
CREATE INDEX idx_stream_tag_tag ON stream_tags(tag);

COMMENT ON TABLE stream_tags IS 'Optional tags for stream discovery and categorization (e.g., "news", "gaming", "tech")';

-- ==================== STREAM EXCLUSIONS ====================
-- Allow streams to explicitly exclude certain boards or users
-- This provides granular control: "Include all tech posts EXCEPT from +spam_board"
CREATE TABLE stream_excluded_boards (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_stream_exclude_board_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_stream_exclude_board_board
        FOREIGN KEY (board_id)
        REFERENCES boards(id)
        ON DELETE CASCADE
);

-- Prevent duplicate exclusions
CREATE UNIQUE INDEX idx_stream_exclude_board_unique ON stream_excluded_boards(stream_id, board_id);

-- Indexes for query filtering
CREATE INDEX idx_stream_exclude_board_stream_id ON stream_excluded_boards(stream_id);
CREATE INDEX idx_stream_exclude_board_board_id ON stream_excluded_boards(board_id);

COMMENT ON TABLE stream_excluded_boards IS 'Boards to exclude from stream even if they match subscription criteria';

-- ==================== STREAM EXCLUDED USERS ====================
-- Allow streams to exclude posts from specific users
CREATE TABLE stream_excluded_users (
    id SERIAL PRIMARY KEY,
    stream_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Foreign keys
    CONSTRAINT fk_stream_exclude_user_stream
        FOREIGN KEY (stream_id)
        REFERENCES streams(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_stream_exclude_user_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Prevent duplicate exclusions
CREATE UNIQUE INDEX idx_stream_exclude_user_unique ON stream_excluded_users(stream_id, user_id);

-- Indexes for query filtering
CREATE INDEX idx_stream_exclude_user_stream_id ON stream_excluded_users(stream_id);
CREATE INDEX idx_stream_exclude_user_user_id ON stream_excluded_users(user_id);

COMMENT ON TABLE stream_excluded_users IS 'Users whose posts to exclude from stream even if they match subscription criteria';

-- ==================== UTILITY FUNCTIONS ====================

-- Function to generate secure share token
CREATE OR REPLACE FUNCTION generate_stream_share_token()
RETURNS VARCHAR(64) AS $$
DECLARE
    token VARCHAR(64);
    token_exists BOOLEAN;
BEGIN
    LOOP
        -- Generate random 64-character token
        token := encode(gen_random_bytes(32), 'hex');

        -- Check if token already exists
        SELECT EXISTS(SELECT 1 FROM streams WHERE share_token = token) INTO token_exists;

        -- Exit loop if token is unique
        EXIT WHEN NOT token_exists;
    END LOOP;

    RETURN token;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION generate_stream_share_token IS 'Generate a cryptographically secure unique share token for private streams';

-- Function to get stream post query (helper for application layer)
-- This returns the list of boards and flairs that should be queried for a stream
CREATE OR REPLACE FUNCTION get_stream_subscription_info(p_stream_id INTEGER)
RETURNS TABLE (
    board_id INTEGER,
    flair_id INTEGER,
    subscription_type VARCHAR(20)
) AS $$
BEGIN
    RETURN QUERY
    -- Get flair-based subscriptions
    SELECT
        sfs.board_id,
        sfs.flair_id,
        'flair'::VARCHAR(20) AS subscription_type
    FROM stream_flair_subscriptions sfs
    WHERE sfs.stream_id = p_stream_id

    UNION ALL

    -- Get board-based subscriptions
    SELECT
        sbs.board_id,
        NULL::INTEGER AS flair_id,
        'board'::VARCHAR(20) AS subscription_type
    FROM stream_board_subscriptions sbs
    WHERE sbs.stream_id = p_stream_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_stream_subscription_info IS 'Returns subscription information for a stream (both flair and board subscriptions)';

-- Function to check if user can access stream
CREATE OR REPLACE FUNCTION can_user_access_stream(p_stream_id INTEGER, p_user_id INTEGER)
RETURNS BOOLEAN AS $$
DECLARE
    stream_record RECORD;
    is_follower BOOLEAN;
BEGIN
    -- Get stream details
    SELECT
        s.creator_id,
        s.is_public,
        s.share_token IS NOT NULL AS is_shared
    INTO stream_record
    FROM streams s
    WHERE s.id = p_stream_id;

    -- Stream doesn't exist
    IF NOT FOUND THEN
        RETURN false;
    END IF;

    -- Creator always has access
    IF stream_record.creator_id = p_user_id THEN
        RETURN true;
    END IF;

    -- Public streams are accessible to everyone
    IF stream_record.is_public THEN
        RETURN true;
    END IF;

    -- Check if user is a follower (has been given access via share link)
    SELECT EXISTS(
        SELECT 1 FROM stream_followers
        WHERE stream_id = p_stream_id AND user_id = p_user_id
    ) INTO is_follower;

    RETURN is_follower;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION can_user_access_stream IS 'Check if a user has permission to access a stream';

-- ==================== ADDITIONAL INDEXES FOR PERFORMANCE ====================

-- Index for efficient "my streams" queries
CREATE INDEX idx_streams_creator_created ON streams(creator_id, created_at DESC);

-- Index for public/discoverable stream listing
CREATE INDEX idx_streams_discoverable ON streams(is_discoverable, created_at DESC)
    WHERE is_discoverable = true;

-- Index for stream search by name (simple text search instead of trigram)
CREATE INDEX idx_streams_name_search ON streams USING btree(lower(name));

COMMENT ON INDEX idx_streams_name_search IS 'B-tree index for case-insensitive stream name search';
