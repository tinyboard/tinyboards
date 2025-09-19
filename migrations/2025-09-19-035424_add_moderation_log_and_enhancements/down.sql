-- Drop indexes for user_ban
DROP INDEX IF EXISTS idx_user_ban_expires_at;
DROP INDEX IF EXISTS idx_user_ban_banned_by;

-- Remove columns from user_ban table
ALTER TABLE user_ban DROP COLUMN IF EXISTS banned_at;
ALTER TABLE user_ban DROP COLUMN IF EXISTS expires_at;
ALTER TABLE user_ban DROP COLUMN IF EXISTS reason;
ALTER TABLE user_ban DROP COLUMN IF EXISTS banned_by;

-- Drop approval status indexes
DROP INDEX IF EXISTS idx_comments_approval_status;
DROP INDEX IF EXISTS idx_posts_approval_status;

-- Remove approval columns from comments
ALTER TABLE comments DROP COLUMN IF EXISTS approved_at;
ALTER TABLE comments DROP COLUMN IF EXISTS approved_by;
ALTER TABLE comments DROP COLUMN IF EXISTS approval_status;

-- Remove approval columns from posts
ALTER TABLE posts DROP COLUMN IF EXISTS approved_at;
ALTER TABLE posts DROP COLUMN IF EXISTS approved_by;
ALTER TABLE posts DROP COLUMN IF EXISTS approval_status;

-- Drop moderation log indexes
DROP INDEX IF EXISTS idx_moderation_log_created_at;
DROP INDEX IF EXISTS idx_moderation_log_board_id;
DROP INDEX IF EXISTS idx_moderation_log_target;
DROP INDEX IF EXISTS idx_moderation_log_action_type;
DROP INDEX IF EXISTS idx_moderation_log_moderator_id;

-- Drop moderation log table
DROP TABLE IF EXISTS moderation_log;
