-- Revert Migration 2/4: Flair Filters and Aggregates
-- This migration removes flair filtering and aggregate tables

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS public.flair_aggregates CASCADE;
DROP TABLE IF EXISTS public.user_flair_filters CASCADE;

-- Drop sequences
DROP SEQUENCE IF EXISTS public.flair_aggregates_id_seq CASCADE;
DROP SEQUENCE IF EXISTS public.user_flair_filters_id_seq CASCADE;
