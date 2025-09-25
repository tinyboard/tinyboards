-- Fix User and Site Aggregates System
-- This migration addresses the missing user aggregates and incorrect site board counts

-- ==================================================================================
-- PART 1: USER AGGREGATES TRIGGER FUNCTIONS
-- ==================================================================================

-- Function to update user aggregate post count when posts are inserted/deleted
CREATE OR REPLACE FUNCTION user_aggregates_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Increment post count for the user
        INSERT INTO user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
        VALUES (NEW.creator_id, 1, 0, 0, 0)
        ON CONFLICT (user_id)
        DO UPDATE SET post_count = user_aggregates.post_count + 1;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement post count for the user
        UPDATE user_aggregates
        SET post_count = GREATEST(0, post_count - 1)
        WHERE user_id = OLD.creator_id;

        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update user aggregate comment count when comments are inserted/deleted
CREATE OR REPLACE FUNCTION user_aggregates_comment_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Increment comment count for the user
        INSERT INTO user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
        VALUES (NEW.creator_id, 0, 0, 1, 0)
        ON CONFLICT (user_id)
        DO UPDATE SET comment_count = user_aggregates.comment_count + 1;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement comment count for the user
        UPDATE user_aggregates
        SET comment_count = GREATEST(0, comment_count - 1)
        WHERE user_id = OLD.creator_id;

        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update user aggregate post score when post aggregates change
CREATE OR REPLACE FUNCTION user_aggregates_post_score()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Add new post score to user aggregates
        UPDATE user_aggregates ua
        SET post_score = ua.post_score + NEW.score
        FROM posts p
        WHERE p.id = NEW.post_id AND ua.user_id = p.creator_id;

        RETURN NEW;
    ELSIF (TG_OP = 'UPDATE') THEN
        -- Update post score difference in user aggregates
        UPDATE user_aggregates ua
        SET post_score = ua.post_score + (NEW.score - OLD.score)
        FROM posts p
        WHERE p.id = NEW.post_id AND ua.user_id = p.creator_id;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Remove post score from user aggregates
        UPDATE user_aggregates ua
        SET post_score = ua.post_score - OLD.score
        FROM posts p
        WHERE p.id = OLD.post_id AND ua.user_id = p.creator_id;

        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update user aggregate comment score when comment aggregates change
CREATE OR REPLACE FUNCTION user_aggregates_comment_score()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Add new comment score to user aggregates
        UPDATE user_aggregates ua
        SET comment_score = ua.comment_score + NEW.score
        FROM comments c
        WHERE c.id = NEW.comment_id AND ua.user_id = c.creator_id;

        RETURN NEW;
    ELSIF (TG_OP = 'UPDATE') THEN
        -- Update comment score difference in user aggregates
        UPDATE user_aggregates ua
        SET comment_score = ua.comment_score + (NEW.score - OLD.score)
        FROM comments c
        WHERE c.id = NEW.comment_id AND ua.user_id = c.creator_id;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Remove comment score from user aggregates
        UPDATE user_aggregates ua
        SET comment_score = ua.comment_score - OLD.score
        FROM comments c
        WHERE c.id = OLD.comment_id AND ua.user_id = c.creator_id;

        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- ==================================================================================
-- PART 2: SITE AGGREGATES TRIGGER FUNCTIONS
-- ==================================================================================

-- Function to update site aggregate board count when boards are inserted/deleted
CREATE OR REPLACE FUNCTION site_aggregates_board_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Increment board count in site aggregates
        UPDATE site_aggregates
        SET boards = boards + 1;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement board count in site aggregates
        UPDATE site_aggregates
        SET boards = GREATEST(0, boards - 1);

        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- ==================================================================================
-- PART 3: CREATE TRIGGERS
-- ==================================================================================

-- User aggregate triggers for post count
CREATE TRIGGER user_aggregates_post_count_trigger
    AFTER INSERT OR DELETE ON posts
    FOR EACH ROW
    EXECUTE FUNCTION user_aggregates_post_count();

-- User aggregate triggers for comment count
CREATE TRIGGER user_aggregates_comment_count_trigger
    AFTER INSERT OR DELETE ON comments
    FOR EACH ROW
    EXECUTE FUNCTION user_aggregates_comment_count();

-- User aggregate triggers for post score (from post_aggregates changes)
CREATE TRIGGER user_aggregates_post_score_trigger
    AFTER INSERT OR UPDATE OR DELETE ON post_aggregates
    FOR EACH ROW
    EXECUTE FUNCTION user_aggregates_post_score();

-- User aggregate triggers for comment score (from comment_aggregates changes)
CREATE TRIGGER user_aggregates_comment_score_trigger
    AFTER INSERT OR UPDATE OR DELETE ON comment_aggregates
    FOR EACH ROW
    EXECUTE FUNCTION user_aggregates_comment_score();

-- Site aggregate trigger for board count
CREATE TRIGGER site_aggregates_board_count_trigger
    AFTER INSERT OR DELETE ON boards
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_board_count();

-- ==================================================================================
-- PART 4: DATA MIGRATION AND RECALCULATION
-- ==================================================================================

-- Create missing user_aggregates records for all users with correct counts
INSERT INTO user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
SELECT
    u.id as user_id,
    COALESCE(p.post_count, 0) as post_count,
    COALESCE(ps.post_score, 0) as post_score,
    COALESCE(c.comment_count, 0) as comment_count,
    COALESCE(cs.comment_score, 0) as comment_score
FROM users u
LEFT JOIN (
    SELECT creator_id, COUNT(*) as post_count
    FROM posts
    GROUP BY creator_id
) p ON u.id = p.creator_id
LEFT JOIN (
    SELECT posts.creator_id, SUM(post_aggregates.score) as post_score
    FROM posts
    JOIN post_aggregates ON posts.id = post_aggregates.post_id
    GROUP BY posts.creator_id
) ps ON u.id = ps.creator_id
LEFT JOIN (
    SELECT creator_id, COUNT(*) as comment_count
    FROM comments
    GROUP BY creator_id
) c ON u.id = c.creator_id
LEFT JOIN (
    SELECT comments.creator_id, SUM(comment_aggregates.score) as comment_score
    FROM comments
    JOIN comment_aggregates ON comments.id = comment_aggregates.comment_id
    GROUP BY comments.creator_id
) cs ON u.id = cs.creator_id
ON CONFLICT (user_id) DO UPDATE SET
    post_count = EXCLUDED.post_count,
    post_score = EXCLUDED.post_score,
    comment_count = EXCLUDED.comment_count,
    comment_score = EXCLUDED.comment_score;

-- Fix site aggregates board count (should be 1, not 0)
UPDATE site_aggregates
SET boards = (SELECT COUNT(*) FROM boards);
