-- Revert comment reply counts fix
--
-- This reverts the child_count trigger and resets all child counts to 0

-- Remove the trigger
DROP TRIGGER IF EXISTS comment_aggregates_child_count_trigger ON comments;

-- Remove the trigger function
DROP FUNCTION IF EXISTS comment_aggregates_child_count();

-- Reset all child_count values to 0 (original broken state)
UPDATE comment_aggregates SET child_count = 0;