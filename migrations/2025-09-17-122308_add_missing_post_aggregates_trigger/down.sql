-- Remove the post_aggregates trigger
DROP TRIGGER IF EXISTS post_aggregates_post ON posts;
DROP FUNCTION IF EXISTS post_aggregates_post;
