-- Rollback User and Site Aggregates System
-- This migration reverses the aggregate system fixes

-- ==================================================================================
-- PART 1: DROP TRIGGERS
-- ==================================================================================

-- Drop all user aggregate triggers
DROP TRIGGER IF EXISTS user_aggregates_post_count_trigger ON posts;
DROP TRIGGER IF EXISTS user_aggregates_comment_count_trigger ON comments;
DROP TRIGGER IF EXISTS user_aggregates_post_score_trigger ON post_aggregates;
DROP TRIGGER IF EXISTS user_aggregates_comment_score_trigger ON comment_aggregates;

-- Drop site aggregate trigger
DROP TRIGGER IF EXISTS site_aggregates_board_count_trigger ON boards;

-- ==================================================================================
-- PART 2: DROP TRIGGER FUNCTIONS
-- ==================================================================================

-- Drop all user aggregate functions
DROP FUNCTION IF EXISTS user_aggregates_post_count();
DROP FUNCTION IF EXISTS user_aggregates_comment_count();
DROP FUNCTION IF EXISTS user_aggregates_post_score();
DROP FUNCTION IF EXISTS user_aggregates_comment_score();

-- Drop site aggregate function
DROP FUNCTION IF EXISTS site_aggregates_board_count();

-- ==================================================================================
-- PART 3: CLEAR DATA CHANGES
-- ==================================================================================

-- Clear all user_aggregates records (return to broken state)
DELETE FROM user_aggregates;

-- Reset site aggregates board count to broken state (0)
UPDATE site_aggregates SET boards = 0;
