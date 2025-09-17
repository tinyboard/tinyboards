-- Remove old ActivityPub field
ALTER TABLE emoji DROP COLUMN IF EXISTS local_site_id;

-- Add new board and management fields
ALTER TABLE emoji ADD COLUMN board_id INT4 REFERENCES boards(id);
ALTER TABLE emoji ADD COLUMN created_by_user_id INT4;
ALTER TABLE emoji ADD COLUMN is_active BOOLEAN DEFAULT true NOT NULL;
ALTER TABLE emoji ADD COLUMN usage_count INT4 DEFAULT 0 NOT NULL;
ALTER TABLE emoji ADD COLUMN emoji_scope VARCHAR(10) DEFAULT 'site' NOT NULL
    CHECK (emoji_scope IN ('site', 'board'));

-- Set created_by_user_id to first admin for existing emojis (if any exist)
UPDATE emoji SET created_by_user_id = (
    SELECT id FROM users WHERE admin_level > 0 ORDER BY id LIMIT 1
) WHERE created_by_user_id IS NULL;

-- Handle case where no admins exist - use first user
UPDATE emoji SET created_by_user_id = (
    SELECT id FROM users ORDER BY id LIMIT 1
) WHERE created_by_user_id IS NULL;

-- Make created_by_user_id NOT NULL after populating
ALTER TABLE emoji ALTER COLUMN created_by_user_id SET NOT NULL;
ALTER TABLE emoji ADD CONSTRAINT fk_emoji_created_by_user
    FOREIGN KEY (created_by_user_id) REFERENCES users(id);

-- Create indexes for performance
CREATE INDEX idx_emoji_board_id ON emoji(board_id) WHERE board_id IS NOT NULL;
CREATE INDEX idx_emoji_shortcode ON emoji(shortcode);
CREATE INDEX idx_emoji_scope_active ON emoji(emoji_scope, is_active) WHERE is_active = true;
CREATE UNIQUE INDEX idx_emoji_shortcode_scope ON emoji(shortcode, COALESCE(board_id, 0));
CREATE INDEX idx_emoji_usage ON emoji(usage_count DESC);
CREATE INDEX idx_emoji_created_by ON emoji(created_by_user_id);