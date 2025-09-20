-- This file should undo anything in `up.sql`
-- Drop the ranking functions added in the up migration

DROP FUNCTION IF EXISTS public.hot_rank(score numeric, published timestamp without time zone);
DROP FUNCTION IF EXISTS public.hot_rank(score numeric, published timestamp without time zone, now_timestamp timestamp without time zone);
DROP FUNCTION IF EXISTS public.controversy_rank(upvotes numeric, downvotes numeric, published timestamp without time zone);
DROP FUNCTION IF EXISTS public.scaled_rank(score numeric, published timestamp without time zone, users_active_month numeric);
DROP FUNCTION IF EXISTS public.generate_unique_changeme();
