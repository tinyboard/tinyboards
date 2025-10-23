-- Standardize remaining tables to use creation_date instead of created_at

-- Content uploads
ALTER TABLE content_uploads RENAME COLUMN created_at TO creation_date;

-- Moderation log
ALTER TABLE moderation_log RENAME COLUMN created_at TO creation_date;

-- Stream excluded boards
ALTER TABLE stream_excluded_boards RENAME COLUMN created_at TO creation_date;

-- Stream excluded users
ALTER TABLE stream_excluded_users RENAME COLUMN created_at TO creation_date;

-- Stream tags
ALTER TABLE stream_tags RENAME COLUMN created_at TO creation_date;
