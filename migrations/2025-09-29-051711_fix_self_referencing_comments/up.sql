-- Fix self-referencing comments (comments that have themselves as parent)
-- This is data corruption that breaks the comment tree structure

-- First, let's see what we're dealing with
DO $$
DECLARE
    self_ref_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO self_ref_count
    FROM comments
    WHERE id = parent_id;

    RAISE NOTICE 'Found % self-referencing comments', self_ref_count;
END $$;

-- Fix self-referencing comments by setting their parent_id to NULL
-- (making them top-level comments instead of replies)
UPDATE comments
SET parent_id = NULL
WHERE id = parent_id;

-- Now recalculate child_count values to fix the reply counts
-- Reset all child_count values to 0 first
UPDATE comment_aggregates SET child_count = 0;

-- Recalculate child_count for all parent comments
UPDATE comment_aggregates
SET child_count = (
    SELECT COUNT(*)
    FROM comments
    WHERE comments.parent_id = comment_aggregates.comment_id
);

-- Show the results
DO $$
DECLARE
    fixed_count INTEGER;
    total_replies INTEGER;
BEGIN
    SELECT COUNT(*) INTO fixed_count
    FROM comments
    WHERE parent_id IS NULL AND id IN (
        SELECT DISTINCT parent_id
        FROM comments
        WHERE parent_id IS NOT NULL
    );

    SELECT COUNT(*) INTO total_replies
    FROM comments
    WHERE parent_id IS NOT NULL;

    RAISE NOTICE 'Fixed self-referencing comments';
    RAISE NOTICE 'Total reply comments: %', total_replies;
    RAISE NOTICE 'Comments with replies: %', (
        SELECT COUNT(*)
        FROM comment_aggregates
        WHERE child_count > 0
    );
END $$;
