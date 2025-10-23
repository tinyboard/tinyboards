-- Revert Migration 3/4: Flair Triggers and Functions
-- This migration removes all flair-related triggers and functions

-- Drop triggers on post_aggregates
DROP TRIGGER IF EXISTS update_flair_aggregates_on_post_score_trigger ON public.post_aggregates CASCADE;

-- Drop triggers on user_flairs
DROP TRIGGER IF EXISTS update_flair_aggregates_user_trigger ON public.user_flairs CASCADE;

-- Drop triggers on post_flairs
DROP TRIGGER IF EXISTS update_flair_aggregates_post_trigger ON public.post_flairs CASCADE;
DROP TRIGGER IF EXISTS update_flair_template_usage_post ON public.post_flairs CASCADE;

-- Drop triggers on user_flairs (usage tracking)
DROP TRIGGER IF EXISTS update_flair_template_usage_user ON public.user_flairs CASCADE;

-- Drop triggers on flair_templates
DROP TRIGGER IF EXISTS flair_aggregates_template_trigger ON public.flair_templates CASCADE;

-- Drop updated_at triggers
DROP TRIGGER IF EXISTS set_updated_at_flair_aggregates ON public.flair_aggregates CASCADE;
DROP TRIGGER IF EXISTS set_updated_at_user_flair_filters ON public.user_flair_filters CASCADE;
DROP TRIGGER IF EXISTS set_updated_at_flair_templates ON public.flair_templates CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS public.update_flair_aggregates_on_post_score() CASCADE;
DROP FUNCTION IF EXISTS public.update_flair_aggregates_user() CASCADE;
DROP FUNCTION IF EXISTS public.update_flair_aggregates_post() CASCADE;
DROP FUNCTION IF EXISTS public.update_flair_template_usage() CASCADE;
DROP FUNCTION IF EXISTS public.flair_aggregates_template() CASCADE;
