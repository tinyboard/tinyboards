-- Rollback Migration 2: Drop stream subscription tables

-- Drop indexes for stream_followers
DROP INDEX IF EXISTS idx_stream_follower_navbar;
DROP INDEX IF EXISTS idx_stream_follower_user_id;
DROP INDEX IF EXISTS idx_stream_follower_stream_id;
DROP INDEX IF EXISTS idx_stream_follower_unique;

-- Drop indexes for stream_board_subscriptions
DROP INDEX IF EXISTS idx_stream_board_board_id;
DROP INDEX IF EXISTS idx_stream_board_stream_id;
DROP INDEX IF EXISTS idx_stream_board_unique;

-- Drop indexes for stream_flair_subscriptions
DROP INDEX IF EXISTS idx_stream_flair_flair_id;
DROP INDEX IF EXISTS idx_stream_flair_board_id;
DROP INDEX IF EXISTS idx_stream_flair_stream_id;
DROP INDEX IF EXISTS idx_stream_flair_unique;

-- Drop tables
DROP TABLE IF EXISTS stream_followers CASCADE;
DROP TABLE IF EXISTS stream_board_subscriptions CASCADE;
DROP TABLE IF EXISTS stream_flair_subscriptions CASCADE;
