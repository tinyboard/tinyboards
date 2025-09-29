-- Fix Reputation Calculation: Exclude Self-Votes
--
-- PROBLEM: Users gain reputation from automatically upvoting their own posts and comments
-- SOLUTION: Modify the user_aggregates calculation to exclude votes where the voter is the content creator
--
-- This migration:
-- 1. Updates the trigger functions to exclude self-votes from user reputation
-- 2. Recalculates existing user reputation scores excluding self-votes

-- ========================================================================================
-- SECTION 1: UPDATE TRIGGER FUNCTIONS TO EXCLUDE SELF-VOTES FROM REPUTATION
-- ========================================================================================

-- Function to update post_aggregates when post_votes change (with self-vote exclusion for reputation)
CREATE OR REPLACE FUNCTION post_aggregates_vote_count()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    post_id_ integer;
    new_score bigint;
    new_upvotes bigint;
    new_downvotes bigint;
    old_user_id integer;
    new_user_id integer;
    old_score integer;
    new_score_diff integer;
    content_creator_id integer;
    old_is_self_vote boolean;
    new_is_self_vote boolean;
BEGIN
    -- Handle INSERT and UPDATE cases
    IF (TG_OP = 'INSERT') THEN
        post_id_ := NEW.post_id;
        new_user_id := NEW.user_id;
        new_score_diff := NEW.score;
    ELSIF (TG_OP = 'UPDATE') THEN
        post_id_ := NEW.post_id;
        old_user_id := OLD.user_id;
        new_user_id := NEW.user_id;
        old_score := OLD.score;
        new_score_diff := NEW.score - OLD.score;
    ELSIF (TG_OP = 'DELETE') THEN
        post_id_ := OLD.post_id;
        old_user_id := OLD.user_id;
        new_score_diff := -OLD.score;
    END IF;

    -- Get the content creator ID
    SELECT creator_id INTO content_creator_id FROM posts WHERE id = post_id_;

    -- Calculate new aggregates for the post (all votes count for post aggregates)
    SELECT
        COALESCE(SUM(score), 0),
        COALESCE(SUM(CASE WHEN score > 0 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN score < 0 THEN 1 ELSE 0 END), 0)
    INTO new_score, new_upvotes, new_downvotes
    FROM post_votes
    WHERE post_id = post_id_;

    -- Update post_aggregates
    UPDATE post_aggregates
    SET
        score = new_score,
        upvotes = new_upvotes,
        downvotes = new_downvotes
    WHERE post_aggregates.post_id = post_id_;

    -- Update user_aggregates for post scores (EXCLUDING SELF-VOTES)
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') THEN
        -- Check if this is a self-vote
        new_is_self_vote := (new_user_id = content_creator_id);

        -- Only update reputation if it's NOT a self-vote
        IF new_user_id IS NOT NULL AND NOT new_is_self_vote THEN
            UPDATE user_aggregates
            SET post_score = post_score + new_score_diff
            WHERE user_id = content_creator_id;
        END IF;
    END IF;

    -- Handle UPDATE case - remove old user's contribution (if not self-vote)
    IF (TG_OP = 'UPDATE' AND old_user_id != new_user_id) THEN
        old_is_self_vote := (old_user_id = content_creator_id);

        IF NOT old_is_self_vote THEN
            UPDATE user_aggregates
            SET post_score = post_score - old_score
            WHERE user_id = content_creator_id;
        END IF;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Update user's score for deleted vote (if not self-vote)
        old_is_self_vote := (old_user_id = content_creator_id);

        IF NOT old_is_self_vote THEN
            UPDATE user_aggregates
            SET post_score = post_score + new_score_diff
            WHERE user_id = content_creator_id;
        END IF;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- Function to update comment_aggregates when comment_votes change (with self-vote exclusion for reputation)
CREATE OR REPLACE FUNCTION comment_aggregates_vote_count()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    comment_id_ integer;
    new_score bigint;
    new_upvotes bigint;
    new_downvotes bigint;
    old_user_id integer;
    new_user_id integer;
    old_score integer;
    new_score_diff integer;
    content_creator_id integer;
    old_is_self_vote boolean;
    new_is_self_vote boolean;
BEGIN
    -- Handle INSERT and UPDATE cases
    IF (TG_OP = 'INSERT') THEN
        comment_id_ := NEW.comment_id;
        new_user_id := NEW.user_id;
        new_score_diff := NEW.score;
    ELSIF (TG_OP = 'UPDATE') THEN
        comment_id_ := NEW.comment_id;
        old_user_id := OLD.user_id;
        new_user_id := NEW.user_id;
        old_score := OLD.score;
        new_score_diff := NEW.score - OLD.score;
    ELSIF (TG_OP = 'DELETE') THEN
        comment_id_ := OLD.comment_id;
        old_user_id := OLD.user_id;
        new_score_diff := -OLD.score;
    END IF;

    -- Get the content creator ID
    SELECT creator_id INTO content_creator_id FROM comments WHERE id = comment_id_;

    -- Calculate new aggregates for the comment (all votes count for comment aggregates)
    SELECT
        COALESCE(SUM(score), 0),
        COALESCE(SUM(CASE WHEN score > 0 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN score < 0 THEN 1 ELSE 0 END), 0)
    INTO new_score, new_upvotes, new_downvotes
    FROM comment_votes
    WHERE comment_id = comment_id_;

    -- Update comment_aggregates
    UPDATE comment_aggregates
    SET
        score = new_score,
        upvotes = new_upvotes,
        downvotes = new_downvotes
    WHERE comment_aggregates.comment_id = comment_id_;

    -- Update user_aggregates for comment scores (EXCLUDING SELF-VOTES)
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') THEN
        -- Check if this is a self-vote
        new_is_self_vote := (new_user_id = content_creator_id);

        -- Only update reputation if it's NOT a self-vote
        IF new_user_id IS NOT NULL AND NOT new_is_self_vote THEN
            UPDATE user_aggregates
            SET comment_score = comment_score + new_score_diff
            WHERE user_id = content_creator_id;
        END IF;
    END IF;

    -- Handle UPDATE case - remove old user's contribution (if not self-vote)
    IF (TG_OP = 'UPDATE' AND old_user_id != new_user_id) THEN
        old_is_self_vote := (old_user_id = content_creator_id);

        IF NOT old_is_self_vote THEN
            UPDATE user_aggregates
            SET comment_score = comment_score - old_score
            WHERE user_id = content_creator_id;
        END IF;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Update user's score for deleted vote (if not self-vote)
        old_is_self_vote := (old_user_id = content_creator_id);

        IF NOT old_is_self_vote THEN
            UPDATE user_aggregates
            SET comment_score = comment_score + new_score_diff
            WHERE user_id = content_creator_id;
        END IF;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- ========================================================================================
-- SECTION 2: RECALCULATE EXISTING USER REPUTATION EXCLUDING SELF-VOTES
-- ========================================================================================

-- Recalculate user_aggregates post_score from post votes (EXCLUDING SELF-VOTES)
UPDATE user_aggregates
SET post_score = COALESCE(post_score_data.total_score, 0)
FROM (
    SELECT
        p.creator_id as user_id,
        SUM(pv.score) as total_score
    FROM posts p
    LEFT JOIN post_votes pv ON p.id = pv.post_id
    WHERE pv.user_id != p.creator_id  -- EXCLUDE SELF-VOTES
    GROUP BY p.creator_id
) as post_score_data
WHERE user_aggregates.user_id = post_score_data.user_id;

-- Recalculate user_aggregates comment_score from comment votes (EXCLUDING SELF-VOTES)
UPDATE user_aggregates
SET comment_score = COALESCE(comment_score_data.total_score, 0)
FROM (
    SELECT
        c.creator_id as user_id,
        SUM(cv.score) as total_score
    FROM comments c
    LEFT JOIN comment_votes cv ON c.id = cv.comment_id
    WHERE cv.user_id != c.creator_id  -- EXCLUDE SELF-VOTES
    GROUP BY c.creator_id
) as comment_score_data
WHERE user_aggregates.user_id = comment_score_data.user_id;

-- Reset scores to 0 for users who only had self-votes
UPDATE user_aggregates
SET post_score = 0
WHERE user_id NOT IN (
    SELECT DISTINCT p.creator_id
    FROM posts p
    LEFT JOIN post_votes pv ON p.id = pv.post_id
    WHERE pv.user_id != p.creator_id AND pv.user_id IS NOT NULL
);

UPDATE user_aggregates
SET comment_score = 0
WHERE user_id NOT IN (
    SELECT DISTINCT c.creator_id
    FROM comments c
    LEFT JOIN comment_votes cv ON c.id = cv.comment_id
    WHERE cv.user_id != c.creator_id AND cv.user_id IS NOT NULL
);

-- ========================================================================================
-- VERIFICATION AND REPORTING
-- ========================================================================================

-- Log the fix results
DO $$
DECLARE
    self_votes_posts integer;
    self_votes_comments integer;
    users_affected integer;
BEGIN
    -- Count self-votes that were excluded
    SELECT COUNT(*) INTO self_votes_posts
    FROM posts p
    JOIN post_votes pv ON p.id = pv.post_id
    WHERE pv.user_id = p.creator_id;

    SELECT COUNT(*) INTO self_votes_comments
    FROM comments c
    JOIN comment_votes cv ON c.id = cv.comment_id
    WHERE cv.user_id = c.creator_id;

    SELECT COUNT(*) INTO users_affected
    FROM user_aggregates
    WHERE post_score != 0 OR comment_score != 0;

    RAISE NOTICE 'REPUTATION FIX COMPLETED:';
    RAISE NOTICE '- Self-votes on posts excluded from reputation: %', self_votes_posts;
    RAISE NOTICE '- Self-votes on comments excluded from reputation: %', self_votes_comments;
    RAISE NOTICE '- Users with non-zero reputation (after fix): %', users_affected;
    RAISE NOTICE 'Self-votes are now excluded from reputation calculation but still count for post/comment scores.';
END $$;
