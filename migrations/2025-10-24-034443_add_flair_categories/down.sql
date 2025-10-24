-- Rollback: Remove Flair Categories

-- Drop trigger
DROP TRIGGER IF EXISTS trigger_update_flair_category_timestamp ON public.flair_categories;
DROP FUNCTION IF EXISTS update_flair_category_timestamp();

-- Remove category_id from flair_templates
ALTER TABLE public.flair_templates DROP COLUMN IF EXISTS category_id;

-- Drop flair_categories table
DROP TABLE IF EXISTS public.flair_categories CASCADE;
DROP SEQUENCE IF EXISTS public.flair_categories_id_seq;
