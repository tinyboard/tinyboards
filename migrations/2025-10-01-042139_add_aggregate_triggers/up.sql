-- Add missing board aggregate triggers for post and comment counts
-- Board subscriber count trigger already exists from migration 2025-01-01-000003

-- Function to update board aggregate post count when posts are inserted/deleted/updated
CREATE OR REPLACE FUNCTION board_aggregates_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT' AND NEW.is_deleted = false AND NEW.is_removed = false) THEN
        -- Increment post count for the board
        UPDATE board_aggregates
        SET posts = posts + 1
        WHERE board_id = NEW.board_id;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement post count only if post was not deleted/removed
        IF (OLD.is_deleted = false AND OLD.is_removed = false) THEN
            UPDATE board_aggregates
            SET posts = GREATEST(0, posts - 1)
            WHERE board_id = OLD.board_id;
        END IF;

        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        -- Handle status changes (deletion/removal)
        IF (OLD.is_deleted = false AND OLD.is_removed = false) AND (NEW.is_deleted = true OR NEW.is_removed = true) THEN
            -- Post was marked as deleted or removed
            UPDATE board_aggregates
            SET posts = GREATEST(0, posts - 1)
            WHERE board_id = NEW.board_id;
        ELSIF (OLD.is_deleted = true OR OLD.is_removed = true) AND (NEW.is_deleted = false AND NEW.is_removed = false) THEN
            -- Post was restored
            UPDATE board_aggregates
            SET posts = posts + 1
            WHERE board_id = NEW.board_id;
        END IF;

        RETURN NEW;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update board aggregate comment count when comments are inserted/deleted/updated
CREATE OR REPLACE FUNCTION board_aggregates_comment_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT' AND NEW.is_deleted = false AND NEW.is_removed = false) THEN
        -- Increment comment count for the board (via post)
        UPDATE board_aggregates ba
        SET comments = comments + 1
        FROM posts p
        WHERE p.id = NEW.post_id AND ba.board_id = p.board_id;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement comment count only if comment was not deleted/removed
        IF (OLD.is_deleted = false AND OLD.is_removed = false) THEN
            UPDATE board_aggregates ba
            SET comments = GREATEST(0, comments - 1)
            FROM posts p
            WHERE p.id = OLD.post_id AND ba.board_id = p.board_id;
        END IF;

        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        -- Handle status changes (deletion/removal)
        IF (OLD.is_deleted = false AND OLD.is_removed = false) AND (NEW.is_deleted = true OR NEW.is_removed = true) THEN
            -- Comment was marked as deleted or removed
            UPDATE board_aggregates ba
            SET comments = GREATEST(0, comments - 1)
            FROM posts p
            WHERE p.id = NEW.post_id AND ba.board_id = p.board_id;
        ELSIF (OLD.is_deleted = true OR OLD.is_removed = true) AND (NEW.is_deleted = false AND NEW.is_removed = false) THEN
            -- Comment was restored
            UPDATE board_aggregates ba
            SET comments = comments + 1
            FROM posts p
            WHERE p.id = NEW.post_id AND ba.board_id = p.board_id;
        END IF;

        RETURN NEW;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update site aggregate counts when posts/comments are created/deleted/updated
CREATE OR REPLACE FUNCTION site_aggregates_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT' AND NEW.is_deleted = false AND NEW.is_removed = false) THEN
        UPDATE site_aggregates SET posts = posts + 1;
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE' AND OLD.is_deleted = false AND OLD.is_removed = false) THEN
        UPDATE site_aggregates SET posts = GREATEST(0, posts - 1);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        IF (OLD.is_deleted = false AND OLD.is_removed = false) AND (NEW.is_deleted = true OR NEW.is_removed = true) THEN
            UPDATE site_aggregates SET posts = GREATEST(0, posts - 1);
        ELSIF (OLD.is_deleted = true OR OLD.is_removed = true) AND (NEW.is_deleted = false AND NEW.is_removed = false) THEN
            UPDATE site_aggregates SET posts = posts + 1;
        END IF;
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION site_aggregates_comment_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT' AND NEW.is_deleted = false AND NEW.is_removed = false) THEN
        UPDATE site_aggregates SET comments = comments + 1;
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE' AND OLD.is_deleted = false AND OLD.is_removed = false) THEN
        UPDATE site_aggregates SET comments = GREATEST(0, comments - 1);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        IF (OLD.is_deleted = false AND OLD.is_removed = false) AND (NEW.is_deleted = true OR NEW.is_removed = true) THEN
            UPDATE site_aggregates SET comments = GREATEST(0, comments - 1);
        ELSIF (OLD.is_deleted = true OR OLD.is_removed = true) AND (NEW.is_deleted = false AND NEW.is_removed = false) THEN
            UPDATE site_aggregates SET comments = comments + 1;
        END IF;
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION site_aggregates_user_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT' AND NEW.is_deleted = false AND NEW.is_banned = false) THEN
        UPDATE site_aggregates SET users = users + 1;
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE' AND OLD.is_deleted = false AND OLD.is_banned = false) THEN
        UPDATE site_aggregates SET users = GREATEST(0, users - 1);
        RETURN OLD;
    ELSIF (TG_OP = 'UPDATE') THEN
        IF (OLD.is_deleted = false AND OLD.is_banned = false) AND (NEW.is_deleted = true OR NEW.is_banned = true) THEN
            UPDATE site_aggregates SET users = GREATEST(0, users - 1);
        ELSIF (OLD.is_deleted = true OR OLD.is_banned = true) AND (NEW.is_deleted = false AND NEW.is_banned = false) THEN
            UPDATE site_aggregates SET users = users + 1;
        END IF;
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create triggers
CREATE TRIGGER board_aggregates_post_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON posts
    FOR EACH ROW
    EXECUTE FUNCTION board_aggregates_post_count();

CREATE TRIGGER board_aggregates_comment_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON comments
    FOR EACH ROW
    EXECUTE FUNCTION board_aggregates_comment_count();

CREATE TRIGGER site_aggregates_post_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON posts
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_post_count();

CREATE TRIGGER site_aggregates_comment_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON comments
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_comment_count();

CREATE TRIGGER site_aggregates_user_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON users
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_user_count();

-- Fix existing aggregate counts

-- Recalculate post aggregates comment counts
UPDATE post_aggregates pa
SET comments = COALESCE((
    SELECT COUNT(*)
    FROM comments c
    WHERE c.post_id = pa.post_id AND c.is_deleted = false AND c.is_removed = false
), 0);

-- Recalculate comment aggregates child counts
UPDATE comment_aggregates ca
SET child_count = COALESCE((
    SELECT COUNT(*)::int
    FROM comments c
    WHERE c.parent_id = ca.comment_id AND c.is_deleted = false AND c.is_removed = false
), 0);

-- Recalculate board aggregates
UPDATE board_aggregates ba
SET
    posts = COALESCE((SELECT COUNT(*) FROM posts WHERE board_id = ba.board_id AND is_deleted = false AND is_removed = false), 0),
    comments = COALESCE((
        SELECT COUNT(*)
        FROM comments c
        JOIN posts p ON c.post_id = p.id
        WHERE p.board_id = ba.board_id AND c.is_deleted = false AND c.is_removed = false
    ), 0),
    subscribers = COALESCE((SELECT COUNT(*) FROM board_subscriber WHERE board_id = ba.board_id), 0);

-- Recalculate site aggregates
UPDATE site_aggregates SET
    users = (SELECT COUNT(*) FROM users WHERE is_deleted = false AND is_banned = false),
    posts = (SELECT COUNT(*) FROM posts WHERE is_deleted = false AND is_removed = false),
    comments = (SELECT COUNT(*) FROM comments WHERE is_deleted = false AND is_removed = false),
    boards = (SELECT COUNT(*) FROM boards WHERE is_deleted = false AND is_removed = false);
