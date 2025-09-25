-- Rollback vote aggregation triggers
--
-- This removes the vote aggregation triggers and functions,
-- but leaves the recalculated aggregate data intact.
--
-- WARNING: After rolling back, vote changes will NOT update aggregates
-- until triggers are restored.

-- Remove triggers from vote tables
DROP TRIGGER IF EXISTS post_aggregates_vote_count_trigger ON post_votes;
DROP TRIGGER IF EXISTS comment_aggregates_vote_count_trigger ON comment_votes;

-- Remove the trigger functions
DROP FUNCTION IF EXISTS post_aggregates_vote_count();
DROP FUNCTION IF EXISTS comment_aggregates_vote_count();

-- Note: We do NOT reset aggregates to zero since that would lose data.
-- If you need to reset aggregates to zero, run these manually:
--
-- UPDATE post_aggregates SET score = 0, upvotes = 0, downvotes = 0;
-- UPDATE comment_aggregates SET score = 0, upvotes = 0, downvotes = 0;
-- UPDATE user_aggregates SET post_score = 0, comment_score = 0;
