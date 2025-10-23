-- Revert Migration 4/4: Flair Constraints and Validations
-- This migration removes all validation triggers, functions, and CHECK constraints

-- =============================================================================
-- DROP VALIDATION TRIGGERS
-- =============================================================================

-- Drop validation triggers on user_flairs
DROP TRIGGER IF EXISTS validate_user_flair_type_trigger ON public.user_flairs CASCADE;
DROP TRIGGER IF EXISTS validate_user_flair_custom_text ON public.user_flairs CASCADE;

-- Drop validation triggers on post_flairs
DROP TRIGGER IF EXISTS validate_post_flair_type_trigger ON public.post_flairs CASCADE;
DROP TRIGGER IF EXISTS validate_post_flair_custom_text ON public.post_flairs CASCADE;

-- =============================================================================
-- DROP VALIDATION FUNCTIONS
-- =============================================================================

DROP FUNCTION IF EXISTS public.validate_user_flair_type() CASCADE;
DROP FUNCTION IF EXISTS public.validate_post_flair_type() CASCADE;
DROP FUNCTION IF EXISTS public.validate_flair_custom_text() CASCADE;

-- =============================================================================
-- DROP CHECK CONSTRAINTS ON FLAIR_AGGREGATES
-- =============================================================================

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_avg_post_score_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_total_post_comments_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_usage_last_month_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_usage_last_week_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_usage_last_day_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_active_user_count_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_user_usage_count_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_post_usage_count_check CASCADE;

ALTER TABLE public.flair_aggregates
    DROP CONSTRAINT IF EXISTS flair_aggregates_total_usage_count_check CASCADE;

-- =============================================================================
-- DROP CHECK CONSTRAINTS ON USER_FLAIR_FILTERS
-- =============================================================================

ALTER TABLE public.user_flair_filters
    DROP CONSTRAINT IF EXISTS user_flair_filters_filter_mode_check CASCADE;

-- =============================================================================
-- DROP CHECK CONSTRAINTS ON USER_FLAIRS
-- =============================================================================

ALTER TABLE public.user_flairs
    DROP CONSTRAINT IF EXISTS user_flairs_approval_consistency_check CASCADE;

ALTER TABLE public.user_flairs
    DROP CONSTRAINT IF EXISTS user_flairs_custom_background_color_check CASCADE;

ALTER TABLE public.user_flairs
    DROP CONSTRAINT IF EXISTS user_flairs_custom_text_color_check CASCADE;

ALTER TABLE public.user_flairs
    DROP CONSTRAINT IF EXISTS user_flairs_custom_text_length_check CASCADE;

-- =============================================================================
-- DROP CHECK CONSTRAINTS ON POST_FLAIRS
-- =============================================================================

ALTER TABLE public.post_flairs
    DROP CONSTRAINT IF EXISTS post_flairs_custom_background_color_check CASCADE;

ALTER TABLE public.post_flairs
    DROP CONSTRAINT IF EXISTS post_flairs_custom_text_color_check CASCADE;

ALTER TABLE public.post_flairs
    DROP CONSTRAINT IF EXISTS post_flairs_custom_text_length_check CASCADE;

-- =============================================================================
-- DROP CHECK CONSTRAINTS ON FLAIR_TEMPLATES
-- =============================================================================

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_style_config_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_usage_count_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_display_order_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_max_text_length_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_background_color_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_text_color_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_text_display_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_template_name_check CASCADE;

ALTER TABLE public.flair_templates
    DROP CONSTRAINT IF EXISTS flair_templates_flair_type_check CASCADE;
