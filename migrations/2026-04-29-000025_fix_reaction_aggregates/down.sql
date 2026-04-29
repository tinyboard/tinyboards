-- Restore the original (buggy) trigger and non-unique indexes.
DROP TRIGGER IF EXISTS reaction_aggregates_on_reaction ON reactions;
DROP FUNCTION IF EXISTS trg_reaction_aggregates() CASCADE;
DROP INDEX IF EXISTS idx_reaction_aggregates_post;
DROP INDEX IF EXISTS idx_reaction_aggregates_comment;

CREATE INDEX idx_reaction_aggregates_post ON reaction_aggregates (post_id, emoji)
    WHERE post_id IS NOT NULL;
CREATE INDEX idx_reaction_aggregates_comment ON reaction_aggregates (comment_id, emoji)
    WHERE comment_id IS NOT NULL;

CREATE OR REPLACE FUNCTION trg_reaction_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO reaction_aggregates (post_id, comment_id, emoji, count)
        VALUES (NEW.post_id, NEW.comment_id, NEW.emoji, 1)
        ON CONFLICT DO NOTHING;

        UPDATE reaction_aggregates
        SET count = count + 1
        WHERE emoji = NEW.emoji
          AND (post_id = NEW.post_id OR (post_id IS NULL AND NEW.post_id IS NULL))
          AND (comment_id = NEW.comment_id OR (comment_id IS NULL AND NEW.comment_id IS NULL));

    ELSIF TG_OP = 'DELETE' THEN
        UPDATE reaction_aggregates
        SET count = count - 1
        WHERE emoji = OLD.emoji
          AND (post_id = OLD.post_id OR (post_id IS NULL AND OLD.post_id IS NULL))
          AND (comment_id = OLD.comment_id OR (comment_id IS NULL AND OLD.comment_id IS NULL));

        DELETE FROM reaction_aggregates
        WHERE count <= 0
          AND emoji = OLD.emoji
          AND (post_id = OLD.post_id OR (post_id IS NULL AND OLD.post_id IS NULL))
          AND (comment_id = OLD.comment_id OR (comment_id IS NULL AND OLD.comment_id IS NULL));
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reaction_aggregates_on_reaction
    AFTER INSERT OR DELETE ON reactions
    FOR EACH ROW EXECUTE FUNCTION trg_reaction_aggregates();
