-- TinyBoards Critical Fix: Vote Aggregation System
--
-- PROBLEM: Vote tables exist and votes are stored, but aggregates never update
-- because the critical triggers that update aggregates when votes change are MISSING.
--
-- This migration:
-- 1. Creates missing trigger functions to update aggregates when votes change
-- 2. Installs triggers on post_votes and comment_votes tables
-- 3. Recalculates ALL existing aggregate data from current vote state
-- 4. Updates related aggregates (board_aggregates, site_aggregates, user_aggregates)

-- ========================================================================================
-- SECTION 1: CREATE TRIGGER FUNCTIONS FOR VOTE AGGREGATION
-- ========================================================================================

-- Function to update post_aggregates when post_votes change
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

    -- Calculate new aggregates for the post
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

    -- Update user_aggregates for post scores
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') THEN
        -- Update new user's score
        IF new_user_id IS NOT NULL THEN
            UPDATE user_aggregates
            SET post_score = post_score + new_score_diff
            WHERE user_id = (SELECT creator_id FROM posts WHERE id = post_id_);
        END IF;
    END IF;

    -- Handle UPDATE case - remove old user's contribution
    IF (TG_OP = 'UPDATE' AND old_user_id != new_user_id) THEN
        UPDATE user_aggregates
        SET post_score = post_score - old_score
        WHERE user_id = (SELECT creator_id FROM posts WHERE id = post_id_);
    ELSIF (TG_OP = 'DELETE') THEN
        -- Update user's score for deleted vote
        UPDATE user_aggregates
        SET post_score = post_score + new_score_diff
        WHERE user_id = (SELECT creator_id FROM posts WHERE id = post_id_);
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- Function to update comment_aggregates when comment_votes change
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

    -- Calculate new aggregates for the comment
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

    -- Update user_aggregates for comment scores
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') THEN
        -- Update new user's score
        IF new_user_id IS NOT NULL THEN
            UPDATE user_aggregates
            SET comment_score = comment_score + new_score_diff
            WHERE user_id = (SELECT creator_id FROM comments WHERE id = comment_id_);
        END IF;
    END IF;

    -- Handle UPDATE case - remove old user's contribution
    IF (TG_OP = 'UPDATE' AND old_user_id != new_user_id) THEN
        UPDATE user_aggregates
        SET comment_score = comment_score - old_score
        WHERE user_id = (SELECT creator_id FROM comments WHERE id = comment_id_);
    ELSIF (TG_OP = 'DELETE') THEN
        -- Update user's score for deleted vote
        UPDATE user_aggregates
        SET comment_score = comment_score + new_score_diff
        WHERE user_id = (SELECT creator_id FROM comments WHERE id = comment_id_);
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$;

-- ========================================================================================
-- SECTION 2: INSTALL TRIGGERS ON VOTE TABLES
-- ========================================================================================

-- Install trigger on post_votes table
DROP TRIGGER IF EXISTS post_aggregates_vote_count_trigger ON post_votes;
CREATE TRIGGER post_aggregates_vote_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON post_votes
    FOR EACH ROW
    EXECUTE FUNCTION post_aggregates_vote_count();

-- Install trigger on comment_votes table
DROP TRIGGER IF EXISTS comment_aggregates_vote_count_trigger ON comment_votes;
CREATE TRIGGER comment_aggregates_vote_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON comment_votes
    FOR EACH ROW
    EXECUTE FUNCTION comment_aggregates_vote_count();

-- ========================================================================================
-- SECTION 3: RECALCULATE ALL EXISTING AGGREGATES FROM CURRENT VOTE DATA
-- ========================================================================================

-- Recalculate post_aggregates from existing post_votes
UPDATE post_aggregates
SET
    score = COALESCE(vote_data.total_score, 0),
    upvotes = COALESCE(vote_data.upvote_count, 0),
    downvotes = COALESCE(vote_data.downvote_count, 0)
FROM (
    SELECT
        pv.post_id,
        SUM(pv.score) as total_score,
        SUM(CASE WHEN pv.score > 0 THEN 1 ELSE 0 END) as upvote_count,
        SUM(CASE WHEN pv.score < 0 THEN 1 ELSE 0 END) as downvote_count
    FROM post_votes pv
    GROUP BY pv.post_id
) as vote_data
WHERE post_aggregates.post_id = vote_data.post_id;

-- Recalculate comment_aggregates from existing comment_votes
UPDATE comment_aggregates
SET
    score = COALESCE(vote_data.total_score, 0),
    upvotes = COALESCE(vote_data.upvote_count, 0),
    downvotes = COALESCE(vote_data.downvotes_count, 0)
FROM (
    SELECT
        cv.comment_id,
        SUM(cv.score) as total_score,
        SUM(CASE WHEN cv.score > 0 THEN 1 ELSE 0 END) as upvote_count,
        SUM(CASE WHEN cv.score < 0 THEN 1 ELSE 0 END) as downvotes_count
    FROM comment_votes cv
    GROUP BY cv.comment_id
) as vote_data
WHERE comment_aggregates.comment_id = vote_data.comment_id;

-- ========================================================================================
-- SECTION 4: RECALCULATE USER AGGREGATE SCORES
-- ========================================================================================

-- Recalculate user_aggregates post_score from post votes
UPDATE user_aggregates
SET post_score = COALESCE(post_score_data.total_score, 0)
FROM (
    SELECT
        p.creator_id as user_id,
        SUM(pv.score) as total_score
    FROM posts p
    LEFT JOIN post_votes pv ON p.id = pv.post_id
    GROUP BY p.creator_id
) as post_score_data
WHERE user_aggregates.user_id = post_score_data.user_id;

-- Recalculate user_aggregates comment_score from comment votes
UPDATE user_aggregates
SET comment_score = COALESCE(comment_score_data.total_score, 0)
FROM (
    SELECT
        c.creator_id as user_id,
        SUM(cv.score) as total_score
    FROM comments c
    LEFT JOIN comment_votes cv ON c.id = cv.comment_id
    GROUP BY c.creator_id
) as comment_score_data
WHERE user_aggregates.user_id = comment_score_data.user_id;

-- ========================================================================================
-- SECTION 5: UPDATE BOARD AND SITE AGGREGATES (Optional consistency improvements)
-- ========================================================================================

-- Update board_aggregates post counts (should be correct, but ensuring consistency)
UPDATE board_aggregates
SET posts = COALESCE(post_count_data.post_count, 0)
FROM (
    SELECT
        board_id,
        COUNT(*) as post_count
    FROM posts
    WHERE is_deleted = false AND is_removed = false
    GROUP BY board_id
) as post_count_data
WHERE board_aggregates.board_id = post_count_data.board_id;

-- Update board_aggregates comment counts
UPDATE board_aggregates
SET comments = COALESCE(comment_count_data.comment_count, 0)
FROM (
    SELECT
        c.board_id,
        COUNT(*) as comment_count
    FROM comments c
    WHERE is_deleted = false AND is_removed = false
    GROUP BY c.board_id
) as comment_count_data
WHERE board_aggregates.board_id = comment_count_data.board_id;

-- Update site_aggregates totals
UPDATE site_aggregates
SET
    posts = (SELECT COUNT(*) FROM posts WHERE is_deleted = false AND is_removed = false),
    comments = (SELECT COUNT(*) FROM comments WHERE is_deleted = false AND is_removed = false),
    users = (SELECT COUNT(*) FROM users WHERE is_deleted = false);

-- ========================================================================================
-- VERIFICATION AND REPORTING
-- ========================================================================================

-- Log the fix results
DO $$
DECLARE
    vote_count integer;
    aggregate_count integer;
    non_zero_scores integer;
BEGIN
    SELECT COUNT(*) INTO vote_count FROM post_votes;
    SELECT COUNT(*) INTO aggregate_count FROM post_aggregates WHERE score > 0 OR upvotes > 0 OR downvotes > 0;
    SELECT COUNT(*) INTO non_zero_scores FROM post_aggregates WHERE score != 0;

    RAISE NOTICE 'VOTE AGGREGATION FIX COMPLETED:';
    RAISE NOTICE '- Total post votes in database: %', vote_count;
    RAISE NOTICE '- Post aggregates with vote data: %', aggregate_count;
    RAISE NOTICE '- Posts with non-zero scores: %', non_zero_scores;

    SELECT COUNT(*) INTO vote_count FROM comment_votes;
    SELECT COUNT(*) INTO aggregate_count FROM comment_aggregates WHERE score > 0 OR upvotes > 0 OR downvotes > 0;

    RAISE NOTICE '- Total comment votes in database: %', vote_count;
    RAISE NOTICE '- Comment aggregates with vote data: %', aggregate_count;
    RAISE NOTICE 'Vote aggregation triggers are now active and will maintain consistency going forward.';
END $$;
