-- This migration fixes data corruption, so reverting would recreate the corruption
-- Instead, we'll just recalculate the child_count values to a known good state

-- Recalculate child_count for all parent comments
UPDATE comment_aggregates
SET child_count = (
    SELECT COUNT(*)
    FROM comments
    WHERE comments.parent_id = comment_aggregates.comment_id
);

-- Note: We don't restore self-referencing comments as that would recreate the data corruption
