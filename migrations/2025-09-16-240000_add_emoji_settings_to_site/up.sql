-- Add emoji-related configuration options to site table
ALTER TABLE site ADD COLUMN emoji_enabled BOOLEAN DEFAULT true NOT NULL;
ALTER TABLE site ADD COLUMN max_emojis_per_post INT4;
ALTER TABLE site ADD COLUMN max_emojis_per_comment INT4;
ALTER TABLE site ADD COLUMN emoji_max_file_size_mb INT4 DEFAULT 2 NOT NULL;
ALTER TABLE site ADD COLUMN board_emojis_enabled BOOLEAN DEFAULT true NOT NULL;