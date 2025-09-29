-- Revert reputation fix: Restore original trigger functions that include self-votes
--
-- This reverts the reputation calculation back to including self-votes in user reputation
-- and restores the original trigger functions from the previous migration

-- ========================================================================================
-- RESTORE ORIGINAL TRIGGER FUNCTIONS (INCLUDE SELF-VOTES)
-- ========================================================================================

-- Restore original post_aggregates_vote_count function (includes self-votes)
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

    -- Update user_aggregates for post scores (INCLUDING ALL VOTES)
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

-- Restore original comment_aggregates_vote_count function (includes self-votes)
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

    -- Update user_aggregates for comment scores (INCLUDING ALL VOTES)
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
-- RECALCULATE USER REPUTATION INCLUDING SELF-VOTES (RESTORE ORIGINAL BEHAVIOR)
-- ========================================================================================

-- Recalculate user_aggregates post_score INCLUDING self-votes
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

-- Recalculate user_aggregates comment_score INCLUDING self-votes
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
