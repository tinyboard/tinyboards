-- Remove emoji-related configuration options from site table
ALTER TABLE site DROP COLUMN IF EXISTS emoji_enabled;
ALTER TABLE site DROP COLUMN IF EXISTS max_emojis_per_post;
ALTER TABLE site DROP COLUMN IF EXISTS max_emojis_per_comment;
ALTER TABLE site DROP COLUMN IF EXISTS emoji_max_file_size_mb;
ALTER TABLE site DROP COLUMN IF EXISTS board_emojis_enabled;