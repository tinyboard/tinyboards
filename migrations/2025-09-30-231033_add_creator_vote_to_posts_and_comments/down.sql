-- Rollback: Remove creator_vote field from posts and comments

-- Drop triggers
DROP TRIGGER IF EXISTS post_creator_vote_update ON post_votes;
DROP TRIGGER IF EXISTS comment_creator_vote_update ON comment_votes;

-- Drop trigger functions
DROP FUNCTION IF EXISTS update_post_creator_vote();
DROP FUNCTION IF EXISTS update_comment_creator_vote();

-- Remove columns
ALTER TABLE posts DROP COLUMN IF EXISTS creator_vote;
ALTER TABLE comments DROP COLUMN IF EXISTS creator_vote;
