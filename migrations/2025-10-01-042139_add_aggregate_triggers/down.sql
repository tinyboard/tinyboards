-- Drop triggers
DROP TRIGGER IF EXISTS board_aggregates_post_count_trigger ON posts;
DROP TRIGGER IF EXISTS board_aggregates_comment_count_trigger ON comments;
DROP TRIGGER IF EXISTS site_aggregates_post_count_trigger ON posts;
DROP TRIGGER IF EXISTS site_aggregates_comment_count_trigger ON comments;
DROP TRIGGER IF EXISTS site_aggregates_user_count_trigger ON users;

-- Drop functions
DROP FUNCTION IF EXISTS board_aggregates_post_count();
DROP FUNCTION IF EXISTS board_aggregates_comment_count();
DROP FUNCTION IF EXISTS site_aggregates_post_count();
DROP FUNCTION IF EXISTS site_aggregates_comment_count();
DROP FUNCTION IF EXISTS site_aggregates_user_count();
