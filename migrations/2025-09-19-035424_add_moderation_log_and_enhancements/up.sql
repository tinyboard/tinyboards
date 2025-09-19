-- Add comprehensive moderation log table
CREATE TABLE moderation_log (
    id SERIAL PRIMARY KEY,
    moderator_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action_type VARCHAR(50) NOT NULL, -- 'ban_user', 'unban_user', 'resolve_report', 'approve_post', 'approve_comment', etc.
    target_type VARCHAR(20) NOT NULL, -- 'user', 'post', 'comment', 'report'
    target_id INTEGER NOT NULL,
    board_id INTEGER REFERENCES boards(id) ON DELETE SET NULL, -- NULL for site-wide actions
    reason TEXT,
    metadata JSONB, -- Additional data like ban duration, original content, etc.
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP -- For temporary bans
);

-- Add indexes for efficient querying
CREATE INDEX idx_moderation_log_moderator_id ON moderation_log(moderator_id);
CREATE INDEX idx_moderation_log_action_type ON moderation_log(action_type);
CREATE INDEX idx_moderation_log_target ON moderation_log(target_type, target_id);
CREATE INDEX idx_moderation_log_board_id ON moderation_log(board_id);
CREATE INDEX idx_moderation_log_created_at ON moderation_log(created_at);

-- Add missing columns to user_ban table for enhanced functionality
ALTER TABLE user_ban ADD COLUMN banned_by INTEGER REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE user_ban ADD COLUMN reason TEXT;
ALTER TABLE user_ban ADD COLUMN expires_at TIMESTAMP;
ALTER TABLE user_ban ADD COLUMN banned_at TIMESTAMP NOT NULL DEFAULT NOW();

-- Add indexes to user_ban for better performance
CREATE INDEX idx_user_ban_banned_by ON user_ban(banned_by);
CREATE INDEX idx_user_ban_expires_at ON user_ban(expires_at);

-- Add approval status columns to posts and comments for moderation queue
ALTER TABLE posts ADD COLUMN approval_status VARCHAR(20) NOT NULL DEFAULT 'approved';
ALTER TABLE posts ADD COLUMN approved_by INTEGER REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE posts ADD COLUMN approved_at TIMESTAMP;

ALTER TABLE comments ADD COLUMN approval_status VARCHAR(20) NOT NULL DEFAULT 'approved';
ALTER TABLE comments ADD COLUMN approved_by INTEGER REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE comments ADD COLUMN approved_at TIMESTAMP;

-- Add indexes for approval status
CREATE INDEX idx_posts_approval_status ON posts(approval_status);
CREATE INDEX idx_comments_approval_status ON comments(approval_status);

-- Add constraint to prevent duplicate active user bans via trigger
-- Note: We'll handle this in application logic instead of a database constraint
-- since PostgreSQL doesn't allow NOW() in index predicates
