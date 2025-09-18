-- Rollback for content and aggregates migration

-- Drop triggers first (including all triggers that use trigger_set_timestamp function)
DROP TRIGGER IF EXISTS post_aggregates_post ON public.posts;
DROP TRIGGER IF EXISTS comment_aggregates_comment ON public.comments;
DROP TRIGGER IF EXISTS board_aggregates_board ON public.boards;
DROP TRIGGER IF EXISTS set_timestamp_posts ON public.posts;
DROP TRIGGER IF EXISTS set_timestamp_comments ON public.comments;
DROP TRIGGER IF EXISTS set_timestamp_users ON public.users;
DROP TRIGGER IF EXISTS set_timestamp_boards ON public.boards;
DROP TRIGGER IF EXISTS set_timestamp_site ON public.site;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS public.site_aggregates CASCADE;
DROP TABLE IF EXISTS public.user_aggregates CASCADE;
DROP TABLE IF EXISTS public.board_aggregates CASCADE;
DROP TABLE IF EXISTS public.comment_aggregates CASCADE;
DROP TABLE IF EXISTS public.post_aggregates CASCADE;
DROP TABLE IF EXISTS public.board_mods CASCADE;
DROP TABLE IF EXISTS public.comment_votes CASCADE;
DROP TABLE IF EXISTS public.post_votes CASCADE;
DROP TABLE IF EXISTS public.comments CASCADE;
DROP TABLE IF EXISTS public.posts CASCADE;

-- Drop sequences
DROP SEQUENCE IF EXISTS public.site_aggregates_id_seq;
DROP SEQUENCE IF EXISTS public.user_aggregates_id_seq;
DROP SEQUENCE IF EXISTS public.board_aggregates_id_seq;
DROP SEQUENCE IF EXISTS public.comment_aggregates_id_seq;
DROP SEQUENCE IF EXISTS public.post_aggregates_id_seq;
DROP SEQUENCE IF EXISTS public.board_mods_id_seq;
DROP SEQUENCE IF EXISTS public.comment_votes_id_seq;
DROP SEQUENCE IF EXISTS public.post_votes_id_seq;
DROP SEQUENCE IF EXISTS public.comments_id_seq;
DROP SEQUENCE IF EXISTS public.posts_id_seq;

-- Drop functions
DROP FUNCTION IF EXISTS public.trigger_set_timestamp();
DROP FUNCTION IF EXISTS public.comment_aggregates_comment();
DROP FUNCTION IF EXISTS public.post_aggregates_post();
DROP FUNCTION IF EXISTS public.site_aggregates_post_insert();
DROP FUNCTION IF EXISTS public.site_aggregates_post_delete();
DROP FUNCTION IF EXISTS public.user_aggregates_post_count();