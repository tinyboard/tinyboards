-- Revert remaining tables back to created_at

ALTER TABLE stream_tags RENAME COLUMN creation_date TO created_at;
ALTER TABLE stream_excluded_users RENAME COLUMN creation_date TO created_at;
ALTER TABLE stream_excluded_boards RENAME COLUMN creation_date TO created_at;
ALTER TABLE moderation_log RENAME COLUMN creation_date TO created_at;
ALTER TABLE content_uploads RENAME COLUMN creation_date TO created_at;
