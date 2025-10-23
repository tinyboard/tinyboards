-- Revert remaining updated fields

ALTER TABLE stream_aggregates RENAME COLUMN updated TO updated_at;
ALTER TABLE user_flair_filters RENAME COLUMN updated TO updated_at;
