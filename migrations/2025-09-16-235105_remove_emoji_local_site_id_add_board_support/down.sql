-- Drop indexes
DROP INDEX IF EXISTS idx_emoji_board_id;
DROP INDEX IF EXISTS idx_emoji_shortcode;
DROP INDEX IF EXISTS idx_emoji_scope_active;
DROP INDEX IF EXISTS idx_emoji_shortcode_scope;
DROP INDEX IF EXISTS idx_emoji_usage;
DROP INDEX IF EXISTS idx_emoji_created_by;

-- Drop constraint
ALTER TABLE emoji DROP CONSTRAINT IF EXISTS fk_emoji_created_by_user;

-- Remove new fields
ALTER TABLE emoji DROP COLUMN IF EXISTS board_id;
ALTER TABLE emoji DROP COLUMN IF EXISTS created_by_user_id;
ALTER TABLE emoji DROP COLUMN IF EXISTS is_active;
ALTER TABLE emoji DROP COLUMN IF EXISTS usage_count;
ALTER TABLE emoji DROP COLUMN IF EXISTS emoji_scope;

-- Add back local_site_id (if needed for rollback)
ALTER TABLE emoji ADD COLUMN local_site_id INT4 DEFAULT 1 NOT NULL;