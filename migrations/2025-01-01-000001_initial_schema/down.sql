-- Rollback for initial schema migration
-- This removes all core tables, functions, and extensions

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS public.boards CASCADE;
DROP TABLE IF EXISTS public.users CASCADE;
DROP TABLE IF EXISTS public.site CASCADE;
DROP TABLE IF EXISTS public.secret CASCADE;
DROP TABLE IF EXISTS public.language CASCADE;

-- Drop sequences
DROP SEQUENCE IF EXISTS public.board_id_seq;
DROP SEQUENCE IF EXISTS public.users_id_seq;
DROP SEQUENCE IF EXISTS public.site_id_seq;
DROP SEQUENCE IF EXISTS public.secret_id_seq;
DROP SEQUENCE IF EXISTS public.language_id_seq;

-- Drop functions
DROP FUNCTION IF EXISTS public.board_aggregates_activity(text);
DROP FUNCTION IF EXISTS public.board_aggregates_board();
DROP FUNCTION IF EXISTS public.board_aggregates_comment_count();
DROP FUNCTION IF EXISTS public.board_aggregates_post_count();
DROP FUNCTION IF EXISTS public.board_aggregates_subscriber_count();
DROP FUNCTION IF EXISTS public.board_mods_delete_update_ranks();
DROP FUNCTION IF EXISTS public.board_mods_insert_set_rank();
DROP FUNCTION IF EXISTS public.set_updated_at();
DROP FUNCTION IF EXISTS public.set_invite_accepted_date();

-- Drop extensions
DROP EXTENSION IF EXISTS pgcrypto;