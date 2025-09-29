-- Exclude Self-Votes from Post and Comment Scores
--
-- PROBLEM: Self-votes are currently included in post and comment scores, allowing users
--          to artificially inflate their content scores by creating posts/comments.
-- SOLUTION: Modify aggregation functions to exclude votes where voter is the content creator.
--
-- This migration:
-- 1. Updates trigger functions to exclude self-votes from post and comment scores
-- 2. Recalculates existing post and comment scores excluding self-votes

-- ========================================================================================
-- SECTION 1: UPDATE TRIGGER FUNCTIONS TO EXCLUDE SELF-VOTES FROM ALL SCORES
-- ========================================================================================

-- Function to update post_aggregates when post_votes change (EXCLUDING SELF-VOTES)
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

    -- Calculate new aggregates for the post (EXCLUDING SELF-VOTES)
    SELECT
        COALESCE(SUM(score), 0),
        COALESCE(SUM(CASE WHEN score > 0 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN score < 0 THEN 1 ELSE 0 END), 0)
    INTO new_score, new_upvotes, new_downvotes
    FROM post_votes
    WHERE post_id = post_id_ AND user_id != content_creator_id;  -- EXCLUDE SELF-VOTES

    -- Update post_aggregates
    UPDATE post_aggregates
    SET
        score = new_score,
        upvotes = new_upvotes,
        downvotes = new_downvotes
    WHERE post_aggregates.post_id = post_id_;

    -- Update user_aggregates for post scores (ALREADY EXCLUDING SELF-VOTES)
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

-- Function to update comment_aggregates when comment_votes change (EXCLUDING SELF-VOTES)
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

    -- Calculate new aggregates for the comment (EXCLUDING SELF-VOTES)
    SELECT
        COALESCE(SUM(score), 0),
        COALESCE(SUM(CASE WHEN score > 0 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN score < 0 THEN 1 ELSE 0 END), 0)
    INTO new_score, new_upvotes, new_downvotes
    FROM comment_votes
    WHERE comment_id = comment_id_ AND user_id != content_creator_id;  -- EXCLUDE SELF-VOTES

    -- Update comment_aggregates
    UPDATE comment_aggregates
    SET
        score = new_score,
        upvotes = new_upvotes,
        downvotes = new_downvotes
    WHERE comment_aggregates.comment_id = comment_id_;

    -- Update user_aggregates for comment scores (ALREADY EXCLUDING SELF-VOTES)
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
-- SECTION 2: RECALCULATE EXISTING POST AND COMMENT SCORES EXCLUDING SELF-VOTES
-- ========================================================================================

-- Recalculate all post_aggregates excluding self-votes
UPDATE post_aggregates
SET
    score = COALESCE(post_score_data.total_score, 0),
    upvotes = COALESCE(post_score_data.total_upvotes, 0),
    downvotes = COALESCE(post_score_data.total_downvotes, 0)
FROM (
    SELECT
        p.id as post_id,
        SUM(pv.score) as total_score,
        SUM(CASE WHEN pv.score > 0 THEN 1 ELSE 0 END) as total_upvotes,
        SUM(CASE WHEN pv.score < 0 THEN 1 ELSE 0 END) as total_downvotes
    FROM posts p
    LEFT JOIN post_votes pv ON p.id = pv.post_id
    WHERE pv.user_id != p.creator_id OR pv.user_id IS NULL  -- EXCLUDE SELF-VOTES
    GROUP BY p.id
) as post_score_data
WHERE post_aggregates.post_id = post_score_data.post_id;

-- Recalculate all comment_aggregates excluding self-votes
UPDATE comment_aggregates
SET
    score = COALESCE(comment_score_data.total_score, 0),
    upvotes = COALESCE(comment_score_data.total_upvotes, 0),
    downvotes = COALESCE(comment_score_data.total_downvotes, 0)
FROM (
    SELECT
        c.id as comment_id,
        SUM(cv.score) as total_score,
        SUM(CASE WHEN cv.score > 0 THEN 1 ELSE 0 END) as total_upvotes,
        SUM(CASE WHEN cv.score < 0 THEN 1 ELSE 0 END) as total_downvotes
    FROM comments c
    LEFT JOIN comment_votes cv ON c.id = cv.comment_id
    WHERE cv.user_id != c.creator_id OR cv.user_id IS NULL  -- EXCLUDE SELF-VOTES
    GROUP BY c.id
) as comment_score_data
WHERE comment_aggregates.comment_id = comment_score_data.comment_id;

-- ========================================================================================
-- VERIFICATION AND REPORTING
-- ========================================================================================

-- Log the fix results
DO $$
DECLARE
    self_votes_posts integer;
    self_votes_comments integer;
    posts_zero_score integer;
    comments_zero_score integer;
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

    -- Count content with zero scores (likely had only self-votes)
    SELECT COUNT(*) INTO posts_zero_score
    FROM post_aggregates
    WHERE score = 0;

    SELECT COUNT(*) INTO comments_zero_score
    FROM comment_aggregates
    WHERE score = 0;

    RAISE NOTICE 'SELF-VOTE EXCLUSION COMPLETED:';
    RAISE NOTICE '- Self-votes on posts excluded from scores: %', self_votes_posts;
    RAISE NOTICE '- Self-votes on comments excluded from scores: %', self_votes_comments;
    RAISE NOTICE '- Posts with zero score (after excluding self-votes): %', posts_zero_score;
    RAISE NOTICE '- Comments with zero score (after excluding self-votes): %', comments_zero_score;
    RAISE NOTICE 'Self-votes are now completely excluded from all scoring calculations.';
END $$;