-- Revert date field naming standardization

-- Flair templates
ALTER TABLE flair_templates RENAME COLUMN creation_date TO created_at;

-- Post flairs
ALTER TABLE post_flairs RENAME COLUMN creation_date TO assigned_at;

-- User flairs
ALTER TABLE user_flairs RENAME COLUMN creation_date TO assigned_at;

-- User flair filters
ALTER TABLE user_flair_filters RENAME COLUMN creation_date TO created_at;

-- Streams
ALTER TABLE streams RENAME COLUMN creation_date TO created_at;

-- Stream followers
ALTER TABLE stream_followers RENAME COLUMN creation_date TO followed_at;

-- Stream board subscriptions
ALTER TABLE stream_board_subscriptions RENAME COLUMN creation_date TO created_at;

-- Stream flair subscriptions
ALTER TABLE stream_flair_subscriptions RENAME COLUMN creation_date TO created_at;
