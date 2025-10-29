-- Revert: Rename max_emoji_count back to max_text_length
ALTER TABLE flair_templates RENAME COLUMN max_emoji_count TO max_text_length;
