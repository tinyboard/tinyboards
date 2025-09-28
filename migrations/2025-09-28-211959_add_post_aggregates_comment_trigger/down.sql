-- Remove the trigger and function for post aggregates comment count
DROP TRIGGER IF EXISTS post_aggregates_comment_count_trigger ON comments;
DROP FUNCTION IF EXISTS public.post_aggregates_comment_count();
