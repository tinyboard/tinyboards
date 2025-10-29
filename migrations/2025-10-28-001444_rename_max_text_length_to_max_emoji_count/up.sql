-- Rename max_text_length to max_emoji_count for clarity
ALTER TABLE flair_templates RENAME COLUMN max_text_length TO max_emoji_count;
