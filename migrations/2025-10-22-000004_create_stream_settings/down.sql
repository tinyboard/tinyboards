-- Rollback Migration 4: Drop stream settings and additional configuration

-- Drop additional indexes
DROP INDEX IF EXISTS idx_streams_name_trgm;
DROP INDEX IF EXISTS idx_streams_discoverable;
DROP INDEX IF EXISTS idx_streams_creator_created;

-- Drop utility functions
DROP FUNCTION IF EXISTS can_user_access_stream(INTEGER, INTEGER);
DROP FUNCTION IF EXISTS get_stream_subscription_info(INTEGER);
DROP FUNCTION IF EXISTS generate_stream_share_token();

-- Drop stream excluded users
DROP INDEX IF EXISTS idx_stream_exclude_user_user_id;
DROP INDEX IF EXISTS idx_stream_exclude_user_stream_id;
DROP INDEX IF EXISTS idx_stream_exclude_user_unique;
DROP TABLE IF EXISTS stream_excluded_users CASCADE;

-- Drop stream excluded boards
DROP INDEX IF EXISTS idx_stream_exclude_board_board_id;
DROP INDEX IF EXISTS idx_stream_exclude_board_stream_id;
DROP INDEX IF EXISTS idx_stream_exclude_board_unique;
DROP TABLE IF EXISTS stream_excluded_boards CASCADE;

-- Drop stream tags
DROP INDEX IF EXISTS idx_stream_tag_tag;
DROP INDEX IF EXISTS idx_stream_tag_stream_id;
DROP INDEX IF EXISTS idx_stream_tag_unique;
DROP TABLE IF EXISTS stream_tags CASCADE;

-- Drop stream view history
DROP INDEX IF EXISTS idx_stream_view_recent;
DROP INDEX IF EXISTS idx_stream_view_user_id;
DROP INDEX IF EXISTS idx_stream_view_stream_id;
DROP TABLE IF EXISTS stream_view_history CASCADE;
