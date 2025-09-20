-- Add missing ranking functions that are used by the application
-- These functions are essential for post and comment sorting algorithms

-- Hot ranking function (2-parameter version)
CREATE OR REPLACE FUNCTION public.hot_rank(score numeric, published timestamp without time zone)
RETURNS integer
LANGUAGE plpgsql IMMUTABLE
AS $$
BEGIN
  RETURN floor(10000*log(greatest(1, score + 3)) / power(((EXTRACT(epoch from now()) - EXTRACT(epoch from published))/3600) + 2, 1.8))::integer;
END;
$$;

-- Hot ranking function (3-parameter version with explicit timestamp)
CREATE OR REPLACE FUNCTION public.hot_rank(score numeric, published timestamp without time zone, now_timestamp timestamp without time zone)
RETURNS integer
LANGUAGE plpgsql IMMUTABLE
AS $$
BEGIN
  RETURN floor(10000*log(greatest(1, score + 3)) / power(((EXTRACT(epoch from now_timestamp) - EXTRACT(epoch from published))/3600) + 2, 1.8))::integer;
END;
$$;

-- Controversy ranking function for sorting by controversial posts
CREATE OR REPLACE FUNCTION public.controversy_rank(upvotes numeric, downvotes numeric, published timestamp without time zone)
RETURNS double precision
LANGUAGE plpgsql IMMUTABLE
AS $$
BEGIN
  IF upvotes <= 0 OR downvotes <= 0 THEN
    RETURN 0;
  END IF;

  RETURN (power(upvotes + downvotes, CASE WHEN upvotes > downvotes THEN downvotes / upvotes::float ELSE upvotes / downvotes::float END)) / power(((EXTRACT(epoch from now()) - EXTRACT(epoch from published))/3600) + 2, 1.8);
END;
$$;

-- Scaled ranking function for adjusting scores based on user activity
CREATE OR REPLACE FUNCTION public.scaled_rank(score numeric, published timestamp without time zone, users_active_month numeric)
RETURNS integer
LANGUAGE plpgsql IMMUTABLE
AS $$
BEGIN
  RETURN floor(hot_rank(score, published) / power(greatest(1, users_active_month), 0.5))::integer;
END;
$$;

-- Utility function for generating unique identifiers
CREATE OR REPLACE FUNCTION public.generate_unique_changeme()
RETURNS text
LANGUAGE plpgsql
AS $$
BEGIN
  RETURN 'changeme_' || extract(epoch from now())::bigint || '_' || floor(random() * 1000000)::text;
END;
$$;
