-- Rollback Migration 3: Drop stream aggregates and triggers

-- Drop helper function
DROP FUNCTION IF EXISTS recalculate_stream_aggregates(INTEGER);

-- Drop triggers
DROP TRIGGER IF EXISTS stream_aggregates_follower_trigger ON stream_followers;
DROP TRIGGER IF EXISTS stream_aggregates_board_subscription_trigger ON stream_board_subscriptions;
DROP TRIGGER IF EXISTS stream_aggregates_flair_subscription_trigger ON stream_flair_subscriptions;
DROP TRIGGER IF EXISTS stream_aggregates_stream_trigger ON streams;

-- Drop trigger functions
DROP FUNCTION IF EXISTS stream_aggregates_follower();
DROP FUNCTION IF EXISTS stream_aggregates_board_subscription();
DROP FUNCTION IF EXISTS stream_aggregates_flair_subscription();
DROP FUNCTION IF EXISTS stream_aggregates_stream();

-- Drop indexes
DROP INDEX IF EXISTS idx_stream_agg_total_subs;
DROP INDEX IF EXISTS idx_stream_agg_follower_count;
DROP INDEX IF EXISTS idx_stream_agg_stream_id;

-- Drop table
DROP TABLE IF EXISTS stream_aggregates CASCADE;
