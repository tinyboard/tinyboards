-- Remove streams feature entirely

DROP TRIGGER IF EXISTS stream_follower_count ON stream_followers;
DROP FUNCTION IF EXISTS trg_stream_follower_count() CASCADE;
DROP TRIGGER IF EXISTS stream_flair_sub_count ON stream_flair_subscriptions;
DROP FUNCTION IF EXISTS trg_stream_flair_sub_count() CASCADE;
DROP TRIGGER IF EXISTS stream_board_sub_count ON stream_board_subscriptions;
DROP FUNCTION IF EXISTS trg_stream_board_sub_count() CASCADE;
DROP TRIGGER IF EXISTS stream_aggregates_on_stream ON streams;
DROP FUNCTION IF EXISTS trg_stream_aggregates_on_stream() CASCADE;

DROP TABLE IF EXISTS stream_aggregates CASCADE;
DROP TABLE IF EXISTS stream_view_history CASCADE;
DROP TABLE IF EXISTS stream_tags CASCADE;
DROP TABLE IF EXISTS stream_excluded_users CASCADE;
DROP TABLE IF EXISTS stream_excluded_boards CASCADE;
DROP TABLE IF EXISTS stream_followers CASCADE;
DROP TABLE IF EXISTS stream_flair_subscriptions CASCADE;
DROP TABLE IF EXISTS stream_board_subscriptions CASCADE;
DROP TABLE IF EXISTS streams CASCADE;
