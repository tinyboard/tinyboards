-- Aggregate tables and their maintenance triggers

-- ============================================================
-- post_aggregates
-- ============================================================

CREATE TABLE post_aggregates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id         UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    board_id        UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    comments        BIGINT NOT NULL DEFAULT 0,
    score           BIGINT NOT NULL DEFAULT 0,
    upvotes         BIGINT NOT NULL DEFAULT 0,
    downvotes       BIGINT NOT NULL DEFAULT 0,
    hot_rank        INT NOT NULL DEFAULT 0,
    hot_rank_active INT NOT NULL DEFAULT 0,
    controversy_rank FLOAT8 NOT NULL DEFAULT 0,
    is_featured_board BOOLEAN NOT NULL DEFAULT false,
    is_featured_local BOOLEAN NOT NULL DEFAULT false,
    newest_comment_time      TIMESTAMPTZ NOT NULL DEFAULT now(),
    newest_comment_time_necro TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT post_aggregates_post_unique UNIQUE (post_id)
);

CREATE INDEX idx_post_aggregates_board_hot ON post_aggregates (board_id, hot_rank DESC);
CREATE INDEX idx_post_aggregates_board_new ON post_aggregates (board_id, created_at DESC);
CREATE INDEX idx_post_aggregates_score ON post_aggregates (score DESC);
CREATE INDEX idx_post_aggregates_hot ON post_aggregates (hot_rank DESC);
CREATE INDEX idx_post_aggregates_controversy ON post_aggregates (controversy_rank DESC);

-- ============================================================
-- comment_aggregates
-- ============================================================

CREATE TABLE comment_aggregates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    comment_id      UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    score           BIGINT NOT NULL DEFAULT 0,
    upvotes         BIGINT NOT NULL DEFAULT 0,
    downvotes       BIGINT NOT NULL DEFAULT 0,
    child_count     INT NOT NULL DEFAULT 0,
    hot_rank        INT NOT NULL DEFAULT 0,
    controversy_rank FLOAT8 NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT comment_aggregates_comment_unique UNIQUE (comment_id)
);

CREATE INDEX idx_comment_aggregates_score ON comment_aggregates (score DESC);
CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank DESC);

-- ============================================================
-- board_aggregates
-- ============================================================

CREATE TABLE board_aggregates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id        UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    subscribers     BIGINT NOT NULL DEFAULT 0,
    posts           BIGINT NOT NULL DEFAULT 0,
    comments        BIGINT NOT NULL DEFAULT 0,
    users_active_day       BIGINT NOT NULL DEFAULT 0,
    users_active_week      BIGINT NOT NULL DEFAULT 0,
    users_active_month     BIGINT NOT NULL DEFAULT 0,
    users_active_half_year BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT board_aggregates_board_unique UNIQUE (board_id)
);

-- ============================================================
-- user_aggregates
-- ============================================================

CREATE TABLE user_aggregates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_count      BIGINT NOT NULL DEFAULT 0,
    post_score      BIGINT NOT NULL DEFAULT 0,
    comment_count   BIGINT NOT NULL DEFAULT 0,
    comment_score   BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT user_aggregates_user_unique UNIQUE (user_id)
);

-- ============================================================
-- site_aggregates
-- ============================================================

CREATE TABLE site_aggregates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    site_id         UUID NOT NULL REFERENCES site(id) ON DELETE CASCADE,
    users           BIGINT NOT NULL DEFAULT 0,
    posts           BIGINT NOT NULL DEFAULT 0,
    comments        BIGINT NOT NULL DEFAULT 0,
    boards          BIGINT NOT NULL DEFAULT 0,
    upvotes         BIGINT NOT NULL DEFAULT 0,
    downvotes       BIGINT NOT NULL DEFAULT 0,
    users_active_day       BIGINT NOT NULL DEFAULT 0,
    users_active_week      BIGINT NOT NULL DEFAULT 0,
    users_active_month     BIGINT NOT NULL DEFAULT 0,
    users_active_half_year BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT site_aggregates_site_unique UNIQUE (site_id)
);

-- ============================================================
-- Trigger: auto-create post_aggregates row on post insert
-- ============================================================

CREATE OR REPLACE FUNCTION trg_post_aggregates_on_post()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO post_aggregates (post_id, board_id, creator_id, created_at)
        VALUES (NEW.id, NEW.board_id, NEW.creator_id, NEW.created_at);
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM post_aggregates WHERE post_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER post_aggregates_on_post
    AFTER INSERT OR DELETE ON posts
    FOR EACH ROW EXECUTE FUNCTION trg_post_aggregates_on_post();

-- ============================================================
-- Trigger: auto-create comment_aggregates row on comment insert
-- ============================================================

CREATE OR REPLACE FUNCTION trg_comment_aggregates_on_comment()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO comment_aggregates (comment_id, created_at)
        VALUES (NEW.id, NEW.created_at);
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM comment_aggregates WHERE comment_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER comment_aggregates_on_comment
    AFTER INSERT OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION trg_comment_aggregates_on_comment();

-- ============================================================
-- Trigger: auto-create board_aggregates row on board insert
-- ============================================================

CREATE OR REPLACE FUNCTION trg_board_aggregates_on_board()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO board_aggregates (board_id) VALUES (NEW.id);
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM board_aggregates WHERE board_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER board_aggregates_on_board
    AFTER INSERT OR DELETE ON boards
    FOR EACH ROW EXECUTE FUNCTION trg_board_aggregates_on_board();

-- ============================================================
-- Trigger: auto-create user_aggregates row on user insert
-- ============================================================

CREATE OR REPLACE FUNCTION trg_user_aggregates_on_user()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO user_aggregates (user_id) VALUES (NEW.id);
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM user_aggregates WHERE user_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_aggregates_on_user
    AFTER INSERT OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION trg_user_aggregates_on_user();

-- ============================================================
-- Trigger: update post counts in board_aggregates
-- ============================================================

CREATE OR REPLACE FUNCTION trg_board_aggregates_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE board_aggregates SET posts = posts + 1 WHERE board_id = NEW.board_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE board_aggregates SET posts = posts - 1 WHERE board_id = OLD.board_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER board_aggregates_post_count
    AFTER INSERT OR DELETE ON posts
    FOR EACH ROW EXECUTE FUNCTION trg_board_aggregates_post_count();

-- ============================================================
-- Trigger: update comment counts in board_aggregates and post_aggregates
-- ============================================================

CREATE OR REPLACE FUNCTION trg_comment_count_on_comment()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE board_aggregates SET comments = comments + 1 WHERE board_id = NEW.board_id;
        UPDATE post_aggregates
        SET comments = comments + 1,
            newest_comment_time = NEW.created_at
        WHERE post_id = NEW.post_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE board_aggregates SET comments = comments - 1 WHERE board_id = OLD.board_id;
        UPDATE post_aggregates SET comments = comments - 1 WHERE post_id = OLD.post_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER comment_count_on_comment
    AFTER INSERT OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION trg_comment_count_on_comment();

-- ============================================================
-- Trigger: update vote aggregates for posts
-- ============================================================

CREATE OR REPLACE FUNCTION trg_post_vote_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE post_aggregates
        SET score = score + NEW.score,
            upvotes = upvotes + CASE WHEN NEW.score = 1 THEN 1 ELSE 0 END,
            downvotes = downvotes + CASE WHEN NEW.score = -1 THEN 1 ELSE 0 END
        WHERE post_id = NEW.post_id;

        -- Update user reputation (exclude self-votes)
        UPDATE user_aggregates
        SET post_score = post_score + NEW.score
        WHERE user_id = (SELECT creator_id FROM posts WHERE id = NEW.post_id)
          AND user_id != NEW.user_id;

    ELSIF TG_OP = 'DELETE' THEN
        UPDATE post_aggregates
        SET score = score - OLD.score,
            upvotes = upvotes - CASE WHEN OLD.score = 1 THEN 1 ELSE 0 END,
            downvotes = downvotes - CASE WHEN OLD.score = -1 THEN 1 ELSE 0 END
        WHERE post_id = OLD.post_id;

        UPDATE user_aggregates
        SET post_score = post_score - OLD.score
        WHERE user_id = (SELECT creator_id FROM posts WHERE id = OLD.post_id)
          AND user_id != OLD.user_id;

    ELSIF TG_OP = 'UPDATE' THEN
        UPDATE post_aggregates
        SET score = score - OLD.score + NEW.score,
            upvotes = upvotes
                - CASE WHEN OLD.score = 1 THEN 1 ELSE 0 END
                + CASE WHEN NEW.score = 1 THEN 1 ELSE 0 END,
            downvotes = downvotes
                - CASE WHEN OLD.score = -1 THEN 1 ELSE 0 END
                + CASE WHEN NEW.score = -1 THEN 1 ELSE 0 END
        WHERE post_id = NEW.post_id;

        UPDATE user_aggregates
        SET post_score = post_score - OLD.score + NEW.score
        WHERE user_id = (SELECT creator_id FROM posts WHERE id = NEW.post_id)
          AND user_id != NEW.user_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER post_vote_aggregates
    AFTER INSERT OR DELETE OR UPDATE ON post_votes
    FOR EACH ROW EXECUTE FUNCTION trg_post_vote_aggregates();

-- ============================================================
-- Trigger: update vote aggregates for comments
-- ============================================================

CREATE OR REPLACE FUNCTION trg_comment_vote_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE comment_aggregates
        SET score = score + NEW.score,
            upvotes = upvotes + CASE WHEN NEW.score = 1 THEN 1 ELSE 0 END,
            downvotes = downvotes + CASE WHEN NEW.score = -1 THEN 1 ELSE 0 END
        WHERE comment_id = NEW.comment_id;

        UPDATE user_aggregates
        SET comment_score = comment_score + NEW.score
        WHERE user_id = (SELECT creator_id FROM comments WHERE id = NEW.comment_id)
          AND user_id != NEW.user_id;

    ELSIF TG_OP = 'DELETE' THEN
        UPDATE comment_aggregates
        SET score = score - OLD.score,
            upvotes = upvotes - CASE WHEN OLD.score = 1 THEN 1 ELSE 0 END,
            downvotes = downvotes - CASE WHEN OLD.score = -1 THEN 1 ELSE 0 END
        WHERE comment_id = OLD.comment_id;

        UPDATE user_aggregates
        SET comment_score = comment_score - OLD.score
        WHERE user_id = (SELECT creator_id FROM comments WHERE id = OLD.comment_id)
          AND user_id != OLD.user_id;

    ELSIF TG_OP = 'UPDATE' THEN
        UPDATE comment_aggregates
        SET score = score - OLD.score + NEW.score,
            upvotes = upvotes
                - CASE WHEN OLD.score = 1 THEN 1 ELSE 0 END
                + CASE WHEN NEW.score = 1 THEN 1 ELSE 0 END,
            downvotes = downvotes
                - CASE WHEN OLD.score = -1 THEN 1 ELSE 0 END
                + CASE WHEN NEW.score = -1 THEN 1 ELSE 0 END
        WHERE comment_id = NEW.comment_id;

        UPDATE user_aggregates
        SET comment_score = comment_score - OLD.score + NEW.score
        WHERE user_id = (SELECT creator_id FROM comments WHERE id = NEW.comment_id)
          AND user_id != NEW.user_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER comment_vote_aggregates
    AFTER INSERT OR DELETE OR UPDATE ON comment_votes
    FOR EACH ROW EXECUTE FUNCTION trg_comment_vote_aggregates();

-- ============================================================
-- Trigger: update user post count
-- ============================================================

CREATE OR REPLACE FUNCTION trg_user_aggregates_post_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE user_aggregates SET post_count = post_count + 1 WHERE user_id = NEW.creator_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE user_aggregates SET post_count = post_count - 1 WHERE user_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_aggregates_post_count
    AFTER INSERT OR DELETE ON posts
    FOR EACH ROW EXECUTE FUNCTION trg_user_aggregates_post_count();

-- ============================================================
-- Trigger: update user comment count
-- ============================================================

CREATE OR REPLACE FUNCTION trg_user_aggregates_comment_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE user_aggregates SET comment_count = comment_count + 1 WHERE user_id = NEW.creator_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE user_aggregates SET comment_count = comment_count - 1 WHERE user_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER user_aggregates_comment_count
    AFTER INSERT OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION trg_user_aggregates_comment_count();

-- ============================================================
-- Trigger: site-level post/comment/board/user counts
-- ============================================================

CREATE OR REPLACE FUNCTION trg_site_aggregates_post()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE site_aggregates SET posts = posts + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE site_aggregates SET posts = posts - 1;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER site_aggregates_post
    AFTER INSERT OR DELETE ON posts
    FOR EACH ROW EXECUTE FUNCTION trg_site_aggregates_post();

CREATE OR REPLACE FUNCTION trg_site_aggregates_comment()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE site_aggregates SET comments = comments + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE site_aggregates SET comments = comments - 1;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER site_aggregates_comment
    AFTER INSERT OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION trg_site_aggregates_comment();

CREATE OR REPLACE FUNCTION trg_site_aggregates_board()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE site_aggregates SET boards = boards + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE site_aggregates SET boards = boards - 1;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER site_aggregates_board
    AFTER INSERT OR DELETE ON boards
    FOR EACH ROW EXECUTE FUNCTION trg_site_aggregates_board();

CREATE OR REPLACE FUNCTION trg_site_aggregates_user()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE site_aggregates SET users = users + 1;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE site_aggregates SET users = users - 1;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER site_aggregates_user
    AFTER INSERT OR DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION trg_site_aggregates_user();

-- ============================================================
-- Activity functions (used by scheduled tasks)
-- ============================================================

CREATE OR REPLACE FUNCTION board_aggregates_activity(i TEXT)
RETURNS TABLE(board_id UUID, count_ BIGINT) AS $$
BEGIN
    RETURN QUERY
    SELECT pa.board_id, COUNT(DISTINCT p.creator_id)
    FROM post_aggregates pa
    JOIN posts p ON p.id = pa.post_id
    WHERE pa.created_at > (now() - i::interval)
    GROUP BY pa.board_id;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION site_aggregates_activity(i TEXT)
RETURNS BIGINT AS $$
DECLARE
    result BIGINT;
BEGIN
    SELECT COUNT(DISTINCT creator_id) INTO result
    FROM posts
    WHERE created_at > (now() - i::interval);
    RETURN result;
END;
$$ LANGUAGE plpgsql;
