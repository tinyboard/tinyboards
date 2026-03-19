DROP TRIGGER IF EXISTS board_aggregates_subscriber_count ON board_subscribers;
DROP FUNCTION IF EXISTS trg_board_aggregates_subscriber_count() CASCADE;

DROP TABLE IF EXISTS user_languages CASCADE;
DROP TABLE IF EXISTS site_languages CASCADE;
DROP TABLE IF EXISTS board_languages CASCADE;
DROP TABLE IF EXISTS user_follows CASCADE;
DROP TABLE IF EXISTS post_hidden CASCADE;
DROP TABLE IF EXISTS comment_saved CASCADE;
DROP TABLE IF EXISTS post_saved CASCADE;
DROP TABLE IF EXISTS board_blocks CASCADE;
DROP TABLE IF EXISTS user_blocks CASCADE;
DROP TABLE IF EXISTS user_bans CASCADE;
DROP TABLE IF EXISTS board_user_bans CASCADE;
DROP TABLE IF EXISTS board_subscribers CASCADE;
