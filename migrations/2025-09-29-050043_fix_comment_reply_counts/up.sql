-- Fix Comment Reply Counts: Add trigger to update child_count in comment_aggregates
--
-- PROBLEM: When comments are added as replies (with parent_id), the parent comment's
-- child_count is not being updated, causing reply counts to show as 0
--
-- SOLUTION: Create a trigger function that updates the parent comment's child_count
-- when replies are added or deleted

-- ========================================================================================
-- CREATE TRIGGER FUNCTION TO UPDATE COMMENT REPLY COUNTS
-- ========================================================================================

-- Function to update parent comment's child_count when replies are added/removed
CREATE OR REPLACE FUNCTION comment_aggregates_child_count()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    -- Handle INSERT case: increment parent's child_count
    IF (TG_OP = 'INSERT') THEN
        -- Only update if this comment has a parent (is a reply)
        IF NEW.parent_id IS NOT NULL THEN
            UPDATE comment_aggregates
            SET child_count = child_count + 1
            WHERE comment_id = NEW.parent_id;
        END IF;
        RETURN NEW;
    END IF;

    -- Handle DELETE case: decrement parent's child_count
    IF (TG_OP = 'DELETE') THEN
        -- Only update if this comment had a parent (was a reply)
        IF OLD.parent_id IS NOT NULL THEN
            UPDATE comment_aggregates
            SET child_count = child_count - 1
            WHERE comment_id = OLD.parent_id;
        END IF;
        RETURN OLD;
    END IF;

    -- Handle UPDATE case: check if parent_id changed
    IF (TG_OP = 'UPDATE') THEN
        -- If parent changed from non-null to null (reply became top-level)
        IF OLD.parent_id IS NOT NULL AND NEW.parent_id IS NULL THEN
            UPDATE comment_aggregates
            SET child_count = child_count - 1
            WHERE comment_id = OLD.parent_id;
        -- If parent changed from null to non-null (top-level became reply)
        ELSIF OLD.parent_id IS NULL AND NEW.parent_id IS NOT NULL THEN
            UPDATE comment_aggregates
            SET child_count = child_count + 1
            WHERE comment_id = NEW.parent_id;
        -- If parent changed from one comment to another
        ELSIF OLD.parent_id IS NOT NULL AND NEW.parent_id IS NOT NULL AND OLD.parent_id != NEW.parent_id THEN
            -- Decrement old parent
            UPDATE comment_aggregates
            SET child_count = child_count - 1
            WHERE comment_id = OLD.parent_id;
            -- Increment new parent
            UPDATE comment_aggregates
            SET child_count = child_count + 1
            WHERE comment_id = NEW.parent_id;
        END IF;
        RETURN NEW;
    END IF;

    RETURN NULL;
END;
$$;

-- Create trigger to call the function
DROP TRIGGER IF EXISTS comment_aggregates_child_count_trigger ON comments;
CREATE TRIGGER comment_aggregates_child_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION comment_aggregates_child_count();

-- ========================================================================================
-- RECALCULATE EXISTING CHILD COUNTS
-- ========================================================================================

-- Reset all child_count values to 0 first
UPDATE comment_aggregates SET child_count = 0;

-- Recalculate child_count for all parent comments
UPDATE comment_aggregates
SET child_count = (
    SELECT COUNT(*)
    FROM comments
    WHERE comments.parent_id = comment_aggregates.comment_id
);

-- ========================================================================================
-- VERIFICATION AND REPORTING
-- ========================================================================================

-- Log the fix results
DO $$
DECLARE
    total_replies integer;
    parents_with_replies integer;
BEGIN
    -- Count total reply comments
    SELECT COUNT(*) INTO total_replies
    FROM comments
    WHERE parent_id IS NOT NULL;

    -- Count parent comments that now have correct child_count
    SELECT COUNT(*) INTO parents_with_replies
    FROM comment_aggregates
    WHERE child_count > 0;

    RAISE NOTICE 'COMMENT REPLY COUNTS FIX COMPLETED:';
    RAISE NOTICE '- Total reply comments found: %', total_replies;
    RAISE NOTICE '- Parent comments with replies: %', parents_with_replies;
    RAISE NOTICE 'Child counts are now properly maintained by triggers.';
END $$;