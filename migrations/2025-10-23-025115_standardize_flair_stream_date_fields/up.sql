-- Standardize date field naming to match existing convention (creation_date, not created_at)

-- Flair templates
ALTER TABLE flair_templates RENAME COLUMN created_at TO creation_date;

-- Post flairs
ALTER TABLE post_flairs RENAME COLUMN assigned_at TO creation_date;

-- User flairs
ALTER TABLE user_flairs RENAME COLUMN assigned_at TO creation_date;

-- User flair filters
ALTER TABLE user_flair_filters RENAME COLUMN created_at TO creation_date;

-- Streams
ALTER TABLE streams RENAME COLUMN created_at TO creation_date;

-- Stream followers (already has followed_at, rename to creation_date)
ALTER TABLE stream_followers RENAME COLUMN followed_at TO creation_date;

-- Stream board subscriptions (already has created_at)
ALTER TABLE stream_board_subscriptions RENAME COLUMN created_at TO creation_date;

-- Stream flair subscriptions (already has created_at)
ALTER TABLE stream_flair_subscriptions RENAME COLUMN created_at TO creation_date;
