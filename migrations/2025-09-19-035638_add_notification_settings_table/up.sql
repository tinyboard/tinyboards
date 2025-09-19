-- Create notification_settings table
CREATE TABLE notification_settings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    comment_replies_enabled BOOLEAN NOT NULL DEFAULT true,
    post_replies_enabled BOOLEAN NOT NULL DEFAULT true,
    mentions_enabled BOOLEAN NOT NULL DEFAULT true,
    post_votes_enabled BOOLEAN NOT NULL DEFAULT false,
    comment_votes_enabled BOOLEAN NOT NULL DEFAULT false,
    private_messages_enabled BOOLEAN NOT NULL DEFAULT true,
    board_invites_enabled BOOLEAN NOT NULL DEFAULT true,
    moderator_actions_enabled BOOLEAN NOT NULL DEFAULT true,
    system_notifications_enabled BOOLEAN NOT NULL DEFAULT true,
    created TIMESTAMP NOT NULL DEFAULT now(),
    updated TIMESTAMP,
    UNIQUE(user_id)
);

-- Create index for faster lookups
CREATE INDEX idx_notification_settings_user_id ON notification_settings(user_id);

-- Create default notification settings for existing users
INSERT INTO notification_settings (user_id, created)
SELECT id, creation_date FROM users
ON CONFLICT (user_id) DO NOTHING;