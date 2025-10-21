-- This file should undo anything in `up.sql`
-- Re-add vote notification fields to notification_settings table
ALTER TABLE notification_settings
    ADD COLUMN IF NOT EXISTS post_votes_enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS comment_votes_enabled BOOLEAN NOT NULL DEFAULT false;
