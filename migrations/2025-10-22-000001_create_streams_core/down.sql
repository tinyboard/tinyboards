-- Rollback Migration 1: Drop streams core table and related objects

-- Drop indexes
DROP INDEX IF EXISTS idx_streams_created_at;
DROP INDEX IF EXISTS idx_streams_is_discoverable;
DROP INDEX IF EXISTS idx_streams_is_public;
DROP INDEX IF EXISTS idx_streams_share_token;
DROP INDEX IF EXISTS idx_streams_slug;
DROP INDEX IF EXISTS idx_streams_creator_id;
DROP INDEX IF EXISTS idx_streams_creator_slug;

-- Drop table
DROP TABLE IF EXISTS streams CASCADE;
