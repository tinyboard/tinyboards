-- Drop triggers
DROP TRIGGER IF EXISTS site_aggregates_post_vote_count_trigger ON post_aggregates;
DROP TRIGGER IF EXISTS site_aggregates_comment_vote_count_trigger ON comment_aggregates;

-- Drop trigger functions
DROP FUNCTION IF EXISTS site_aggregates_post_vote_count();
DROP FUNCTION IF EXISTS site_aggregates_comment_vote_count();

-- Remove upvotes and downvotes columns from site_aggregates
ALTER TABLE site_aggregates DROP COLUMN upvotes;
ALTER TABLE site_aggregates DROP COLUMN downvotes;
