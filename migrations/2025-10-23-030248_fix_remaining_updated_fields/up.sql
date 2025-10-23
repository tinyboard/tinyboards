-- Fix remaining tables that still have updated_at instead of updated

-- Stream aggregates
ALTER TABLE stream_aggregates RENAME COLUMN updated_at TO updated;

-- User flair filters
ALTER TABLE user_flair_filters RENAME COLUMN updated_at TO updated;
