-- Add Creator Vote Field to Posts and Comments
--
-- PROBLEM: The frontend can't accurately display scores for non-authors because it doesn't
--          know what the author's current vote is (upvote, no vote, or downvote).
-- SOLUTION: Add a `creator_vote` column that tracks the author's vote, making it available
--          to all users so they can calculate the accurate displayed score.
--
-- This migration:
-- 1. Adds creator_vote column to posts and comments tables (defaults to 1 = upvote)
-- 2. Populates existing data with actual creator votes from vote tables
-- 3. Updates vote triggers to maintain the creator_vote field

-- ========================================================================================
-- SECTION 1: ADD creator_vote COLUMN TO POSTS AND COMMENTS
-- ========================================================================================

-- Add creator_vote to posts table (default 1 = implicit upvote)
ALTER TABLE posts
ADD COLUMN creator_vote integer DEFAULT 1 NOT NULL;

-- Add creator_vote to comments table (default 1 = implicit upvote)
ALTER TABLE comments
ADD COLUMN creator_vote integer DEFAULT 1 NOT NULL;

-- ========================================================================================
-- SECTION 2: POPULATE EXISTING creator_vote DATA FROM VOTE TABLES
-- ========================================================================================

-- Update posts with actual creator votes (where they exist)
UPDATE posts p
SET creator_vote = COALESCE(pv.score, 1)
FROM post_votes pv
WHERE pv.post_id = p.id
  AND pv.user_id = p.creator_id;

-- Update comments with actual creator votes (where they exist)
UPDATE comments c
SET creator_vote = COALESCE(cv.score, 1)
FROM comment_votes cv
WHERE cv.comment_id = c.id
  AND cv.user_id = c.creator_id;

-- ========================================================================================
-- SECTION 3: UPDATE VOTE TRIGGERS TO MAINTAIN creator_vote FIELD
-- ========================================================================================

-- Function to update posts.creator_vote when post_votes change
CREATE OR REPLACE FUNCTION update_post_creator_vote()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    content_creator_id integer;
BEGIN
    -- Get the creator ID
    IF (TG_OP = 'DELETE') THEN
        SELECT creator_id INTO content_creator_id FROM posts WHERE id = OLD.post_id;

        -- If creator removed their vote, set to 0 (no vote)
        IF OLD.user_id = content_creator_id THEN
            UPDATE posts SET creator_vote = 0 WHERE id = OLD.post_id;
        END IF;
    ELSE
        SELECT creator_id INTO content_creator_id FROM posts WHERE id = NEW.post_id;

        -- If creator is voting, update their vote
        IF NEW.user_id = content_creator_id THEN
            UPDATE posts SET creator_vote = NEW.score WHERE id = NEW.post_id;
        END IF;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- Function to update comments.creator_vote when comment_votes change
CREATE OR REPLACE FUNCTION update_comment_creator_vote()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    content_creator_id integer;
BEGIN
    -- Get the creator ID
    IF (TG_OP = 'DELETE') THEN
        SELECT creator_id INTO content_creator_id FROM comments WHERE id = OLD.comment_id;

        -- If creator removed their vote, set to 0 (no vote)
        IF OLD.user_id = content_creator_id THEN
            UPDATE comments SET creator_vote = 0 WHERE id = OLD.comment_id;
        END IF;
    ELSE
        SELECT creator_id INTO content_creator_id FROM comments WHERE id = NEW.comment_id;

        -- If creator is voting, update their vote
        IF NEW.user_id = content_creator_id THEN
            UPDATE comments SET creator_vote = NEW.score WHERE id = NEW.comment_id;
        END IF;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- Create triggers to call these functions
DROP TRIGGER IF EXISTS post_creator_vote_update ON post_votes;
CREATE TRIGGER post_creator_vote_update
AFTER INSERT OR UPDATE OR DELETE ON post_votes
FOR EACH ROW
EXECUTE FUNCTION update_post_creator_vote();

DROP TRIGGER IF EXISTS comment_creator_vote_update ON comment_votes;
CREATE TRIGGER comment_creator_vote_update
AFTER INSERT OR UPDATE OR DELETE ON comment_votes
FOR EACH ROW
EXECUTE FUNCTION update_comment_creator_vote();

-- ========================================================================================
-- VERIFICATION AND REPORTING
-- ========================================================================================

DO $$
DECLARE
    posts_with_upvote integer;
    posts_with_no_vote integer;
    posts_with_downvote integer;
    comments_with_upvote integer;
    comments_with_no_vote integer;
    comments_with_downvote integer;
BEGIN
    -- Count posts by creator vote
    SELECT COUNT(*) INTO posts_with_upvote FROM posts WHERE creator_vote = 1;
    SELECT COUNT(*) INTO posts_with_no_vote FROM posts WHERE creator_vote = 0;
    SELECT COUNT(*) INTO posts_with_downvote FROM posts WHERE creator_vote = -1;

    -- Count comments by creator vote
    SELECT COUNT(*) INTO comments_with_upvote FROM comments WHERE creator_vote = 1;
    SELECT COUNT(*) INTO comments_with_no_vote FROM comments WHERE creator_vote = 0;
    SELECT COUNT(*) INTO comments_with_downvote FROM comments WHERE creator_vote = -1;

    RAISE NOTICE 'CREATOR_VOTE FIELD ADDED:';
    RAISE NOTICE 'Posts - Upvoted by creator: %, No vote: %, Downvoted: %',
        posts_with_upvote, posts_with_no_vote, posts_with_downvote;
    RAISE NOTICE 'Comments - Upvoted by creator: %, No vote: %, Downvoted: %',
        comments_with_upvote, comments_with_no_vote, comments_with_downvote;
END $$;
