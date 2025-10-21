-- Remove vote notification fields from notification_settings table
ALTER TABLE notification_settings
    DROP COLUMN IF EXISTS post_votes_enabled,
    DROP COLUMN IF EXISTS comment_votes_enabled;

-- Delete any existing vote notifications
DELETE FROM notifications WHERE kind IN ('post_vote', 'comment_vote');
