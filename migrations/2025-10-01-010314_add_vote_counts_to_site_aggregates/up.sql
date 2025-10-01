-- Add upvotes and downvotes columns to site_aggregates
ALTER TABLE site_aggregates ADD COLUMN upvotes BIGINT NOT NULL DEFAULT 0;
ALTER TABLE site_aggregates ADD COLUMN downvotes BIGINT NOT NULL DEFAULT 0;

-- ========================================================================================
-- CREATE TRIGGER FUNCTIONS TO KEEP VOTE COUNTS IN SYNC
-- ========================================================================================

-- Function to update site_aggregates when post_aggregates vote counts change
CREATE OR REPLACE FUNCTION site_aggregates_post_vote_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Add new post votes to site aggregates
        UPDATE site_aggregates
        SET
            upvotes = upvotes + NEW.upvotes,
            downvotes = downvotes + NEW.downvotes;
        RETURN NEW;
    ELSIF (TG_OP = 'UPDATE') THEN
        -- Update site aggregates with the difference
        UPDATE site_aggregates
        SET
            upvotes = upvotes + (NEW.upvotes - OLD.upvotes),
            downvotes = downvotes + (NEW.downvotes - OLD.downvotes);
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Remove deleted post votes from site aggregates
        UPDATE site_aggregates
        SET
            upvotes = GREATEST(0, upvotes - OLD.upvotes),
            downvotes = GREATEST(0, downvotes - OLD.downvotes);
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to update site_aggregates when comment_aggregates vote counts change
CREATE OR REPLACE FUNCTION site_aggregates_comment_vote_count()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Add new comment votes to site aggregates
        UPDATE site_aggregates
        SET
            upvotes = upvotes + NEW.upvotes,
            downvotes = downvotes + NEW.downvotes;
        RETURN NEW;
    ELSIF (TG_OP = 'UPDATE') THEN
        -- Update site aggregates with the difference
        UPDATE site_aggregates
        SET
            upvotes = upvotes + (NEW.upvotes - OLD.upvotes),
            downvotes = downvotes + (NEW.downvotes - OLD.downvotes);
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Remove deleted comment votes from site aggregates
        UPDATE site_aggregates
        SET
            upvotes = GREATEST(0, upvotes - OLD.upvotes),
            downvotes = GREATEST(0, downvotes - OLD.downvotes);
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- ========================================================================================
-- INSTALL TRIGGERS
-- ========================================================================================

-- Trigger for post_aggregates vote changes
DROP TRIGGER IF EXISTS site_aggregates_post_vote_count_trigger ON post_aggregates;
CREATE TRIGGER site_aggregates_post_vote_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON post_aggregates
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_post_vote_count();

-- Trigger for comment_aggregates vote changes
DROP TRIGGER IF EXISTS site_aggregates_comment_vote_count_trigger ON comment_aggregates;
CREATE TRIGGER site_aggregates_comment_vote_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON comment_aggregates
    FOR EACH ROW
    EXECUTE FUNCTION site_aggregates_comment_vote_count();

-- ========================================================================================
-- CALCULATE INITIAL VALUES FROM EXISTING DATA
-- ========================================================================================

-- Calculate initial vote counts from post_aggregates and comment_aggregates
UPDATE site_aggregates
SET
    upvotes = (
        SELECT COALESCE(SUM(upvotes), 0) FROM post_aggregates
    ) + (
        SELECT COALESCE(SUM(upvotes), 0) FROM comment_aggregates
    ),
    downvotes = (
        SELECT COALESCE(SUM(downvotes), 0) FROM post_aggregates
    ) + (
        SELECT COALESCE(SUM(downvotes), 0) FROM comment_aggregates
    );
