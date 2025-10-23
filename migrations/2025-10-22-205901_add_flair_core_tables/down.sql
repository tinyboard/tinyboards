-- Revert Migration 1/4: Core Flair Tables
-- This migration removes all core flair tables and their dependencies

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS public.user_flairs CASCADE;
DROP TABLE IF EXISTS public.post_flairs CASCADE;
DROP TABLE IF EXISTS public.flair_templates CASCADE;

-- Drop sequences
DROP SEQUENCE IF EXISTS public.user_flairs_id_seq CASCADE;
DROP SEQUENCE IF EXISTS public.post_flairs_id_seq CASCADE;
DROP SEQUENCE IF EXISTS public.flair_templates_id_seq CASCADE;
