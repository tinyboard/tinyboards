-- Fix reaction_aggregates trigger: the original trigger used
-- `INSERT ... ON CONFLICT DO NOTHING` followed by an unconditional
-- `UPDATE` increment, but the table had no unique constraint to
-- conflict against. The result was that every new reaction created a
-- new aggregate row AND incremented every existing matching row,
-- inflating counts and producing duplicate rows.

-- 1. Drop the buggy trigger and its function.
DROP TRIGGER IF EXISTS reaction_aggregates_on_reaction ON reactions;
DROP FUNCTION IF EXISTS trg_reaction_aggregates() CASCADE;

-- 2. Drop the non-unique indexes; we'll recreate them as UNIQUE.
DROP INDEX IF EXISTS idx_reaction_aggregates_post;
DROP INDEX IF EXISTS idx_reaction_aggregates_comment;

-- 3. Rebuild aggregates from the reactions table to clean up any
-- inflated/duplicated rows produced by the broken trigger.
TRUNCATE reaction_aggregates;

INSERT INTO reaction_aggregates (post_id, comment_id, emoji, count)
SELECT post_id, NULL::uuid, emoji, COUNT(*)::int
FROM reactions
WHERE post_id IS NOT NULL
GROUP BY post_id, emoji;

INSERT INTO reaction_aggregates (post_id, comment_id, emoji, count)
SELECT NULL::uuid, comment_id, emoji, COUNT(*)::int
FROM reactions
WHERE comment_id IS NOT NULL
GROUP BY comment_id, emoji;

-- 4. Add UNIQUE partial indexes so future ON CONFLICT can target them.
CREATE UNIQUE INDEX idx_reaction_aggregates_post
    ON reaction_aggregates (post_id, emoji)
    WHERE post_id IS NOT NULL;

CREATE UNIQUE INDEX idx_reaction_aggregates_comment
    ON reaction_aggregates (comment_id, emoji)
    WHERE comment_id IS NOT NULL;

-- 5. New trigger: UPDATE first, INSERT only if no row exists. Avoids
-- the "two operations" race and works without referencing a named
-- conflict target (partial indexes can be tricky with ON CONFLICT).
CREATE OR REPLACE FUNCTION trg_reaction_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.post_id IS NOT NULL THEN
            UPDATE reaction_aggregates
            SET count = count + 1
            WHERE post_id = NEW.post_id
              AND emoji = NEW.emoji;
            IF NOT FOUND THEN
                INSERT INTO reaction_aggregates (post_id, comment_id, emoji, count)
                VALUES (NEW.post_id, NULL, NEW.emoji, 1);
            END IF;
        ELSIF NEW.comment_id IS NOT NULL THEN
            UPDATE reaction_aggregates
            SET count = count + 1
            WHERE comment_id = NEW.comment_id
              AND emoji = NEW.emoji;
            IF NOT FOUND THEN
                INSERT INTO reaction_aggregates (post_id, comment_id, emoji, count)
                VALUES (NULL, NEW.comment_id, NEW.emoji, 1);
            END IF;
        END IF;

    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.post_id IS NOT NULL THEN
            UPDATE reaction_aggregates
            SET count = count - 1
            WHERE post_id = OLD.post_id
              AND emoji = OLD.emoji;
            DELETE FROM reaction_aggregates
            WHERE post_id = OLD.post_id
              AND emoji = OLD.emoji
              AND count <= 0;
        ELSIF OLD.comment_id IS NOT NULL THEN
            UPDATE reaction_aggregates
            SET count = count - 1
            WHERE comment_id = OLD.comment_id
              AND emoji = OLD.emoji;
            DELETE FROM reaction_aggregates
            WHERE comment_id = OLD.comment_id
              AND emoji = OLD.emoji
              AND count <= 0;
        END IF;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reaction_aggregates_on_reaction
    AFTER INSERT OR DELETE ON reactions
    FOR EACH ROW EXECUTE FUNCTION trg_reaction_aggregates();
