-- Revert Self-Vote Exclusion from Post and Comment Scores
-- This reverts the changes to include self-votes in scoring again

-- Revert post_aggregates_vote_count function to include self-votes
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

    -- Calculate new aggregates for the post (INCLUDING SELF-VOTES)
    SELECT
        COALESCE(SUM(score), 0),
        COALESCE(SUM(CASE WHEN score > 0 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN score < 0 THEN 1 ELSE 0 END), 0)
    INTO new_score, new_upvotes, new_downvotes
    FROM post_votes
    WHERE post_id = post_id_;  -- INCLUDE ALL VOTES

    -- Update post_aggregates
    UPDATE post_aggregates
    SET
        score = new_score,
        upvotes = new_upvotes,
        downvotes = new_downvotes
    WHERE post_aggregates.post_id = post_id_;

    -- Update user_aggregates for post scores (STILL EXCLUDING SELF-VOTES from reputation)
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') THEN
        new_is_self_vote := (new_user_id = content_creator_id);
        IF new_user_id IS NOT NULL AND NOT new_is_self_vote THEN
            UPDATE user_aggregates
            SET post_score = post_score + new_score_diff
            WHERE user_id = content_creator_id;
        END IF;
    END IF;

    IF (TG_OP = 'UPDATE' AND old_user_id != new_user_id) THEN
        old_is_self_vote := (old_user_id = content_creator_id);
        IF NOT old_is_self_vote THEN
            UPDATE user_aggregates
            SET post_score = post_score - old_score
            WHERE user_id = content_creator_id;
        END IF;
    ELSIF (TG_OP = 'DELETE') THEN
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

-- Revert comment_aggregates_vote_count function to include self-votes
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

    -- Calculate new aggregates for the comment (INCLUDING SELF-VOTES)
    SELECT
        COALESCE(SUM(score), 0),
        COALESCE(SUM(CASE WHEN score > 0 THEN 1 ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN score < 0 THEN 1 ELSE 0 END), 0)
    INTO new_score, new_upvotes, new_downvotes
    FROM comment_votes
    WHERE comment_id = comment_id_;  -- INCLUDE ALL VOTES

    -- Update comment_aggregates
    UPDATE comment_aggregates
    SET
        score = new_score,
        upvotes = new_upvotes,
        downvotes = new_downvotes
    WHERE comment_aggregates.comment_id = comment_id_;

    -- Update user_aggregates for comment scores (STILL EXCLUDING SELF-VOTES from reputation)
    IF (TG_OP = 'INSERT' OR TG_OP = 'UPDATE') THEN
        new_is_self_vote := (new_user_id = content_creator_id);
        IF new_user_id IS NOT NULL AND NOT new_is_self_vote THEN
            UPDATE user_aggregates
            SET comment_score = comment_score + new_score_diff
            WHERE user_id = content_creator_id;
        END IF;
    END IF;

    IF (TG_OP = 'UPDATE' AND old_user_id != new_user_id) THEN
        old_is_self_vote := (old_user_id = content_creator_id);
        IF NOT old_is_self_vote THEN
            UPDATE user_aggregates
            SET comment_score = comment_score - old_score
            WHERE user_id = content_creator_id;
        END IF;
    ELSIF (TG_OP = 'DELETE') THEN
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