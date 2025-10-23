-- Migration 4/4: Flair Constraints and Validations
-- This migration adds CHECK constraints and additional validation rules
-- to ensure data integrity in the flair system

-- =============================================================================
-- CHECK CONSTRAINTS ON FLAIR_TEMPLATES
-- =============================================================================

-- Validate flair_type is either 'post' or 'user'
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_flair_type_check
    CHECK (flair_type IN ('post', 'user'));

-- Validate template_name is not empty
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_template_name_check
    CHECK (length(trim(template_name)) > 0);

-- Validate text_display is not empty
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_text_display_check
    CHECK (length(trim(text_display)) > 0);

-- Validate color codes are hex format (#RRGGBB or #RGB)
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_text_color_check
    CHECK (text_color ~ '^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$');

ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_background_color_check
    CHECK (background_color ~ '^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$');

-- Validate max_text_length is reasonable (between 1 and 200)
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_max_text_length_check
    CHECK (max_text_length > 0 AND max_text_length <= 200);

-- Validate display_order is non-negative
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_display_order_check
    CHECK (display_order >= 0);

-- Validate usage_count is non-negative
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_usage_count_check
    CHECK (usage_count >= 0);

-- Validate style_config is a valid JSON object (not array or primitive)
ALTER TABLE public.flair_templates
    ADD CONSTRAINT flair_templates_style_config_check
    CHECK (jsonb_typeof(style_config) = 'object');

-- =============================================================================
-- CHECK CONSTRAINTS ON POST_FLAIRS
-- =============================================================================

-- Validate custom_text length doesn't exceed 64 characters
ALTER TABLE public.post_flairs
    ADD CONSTRAINT post_flairs_custom_text_length_check
    CHECK (length(custom_text) <= 64);

-- Validate custom colors are hex format
ALTER TABLE public.post_flairs
    ADD CONSTRAINT post_flairs_custom_text_color_check
    CHECK (custom_text_color IS NULL OR custom_text_color ~ '^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$');

ALTER TABLE public.post_flairs
    ADD CONSTRAINT post_flairs_custom_background_color_check
    CHECK (custom_background_color IS NULL OR custom_background_color ~ '^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$');

-- =============================================================================
-- CHECK CONSTRAINTS ON USER_FLAIRS
-- =============================================================================

-- Validate custom_text length doesn't exceed 64 characters
ALTER TABLE public.user_flairs
    ADD CONSTRAINT user_flairs_custom_text_length_check
    CHECK (length(custom_text) <= 64);

-- Validate custom colors are hex format
ALTER TABLE public.user_flairs
    ADD CONSTRAINT user_flairs_custom_text_color_check
    CHECK (custom_text_color IS NULL OR custom_text_color ~ '^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$');

ALTER TABLE public.user_flairs
    ADD CONSTRAINT user_flairs_custom_background_color_check
    CHECK (custom_background_color IS NULL OR custom_background_color ~ '^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$');

-- If approved, must have approved_at and approved_by set
ALTER TABLE public.user_flairs
    ADD CONSTRAINT user_flairs_approval_consistency_check
    CHECK (
        (is_approved = false) OR
        (is_approved = true AND approved_at IS NOT NULL AND approved_by IS NOT NULL)
    );

-- =============================================================================
-- CHECK CONSTRAINTS ON USER_FLAIR_FILTERS
-- =============================================================================

-- Validate filter_mode is either 'include' or 'exclude'
ALTER TABLE public.user_flair_filters
    ADD CONSTRAINT user_flair_filters_filter_mode_check
    CHECK (filter_mode IN ('include', 'exclude'));

-- =============================================================================
-- CHECK CONSTRAINTS ON FLAIR_AGGREGATES
-- =============================================================================

-- All count fields must be non-negative
ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_total_usage_count_check
    CHECK (total_usage_count >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_post_usage_count_check
    CHECK (post_usage_count >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_user_usage_count_check
    CHECK (user_usage_count >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_active_user_count_check
    CHECK (active_user_count >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_usage_last_day_check
    CHECK (usage_last_day >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_usage_last_week_check
    CHECK (usage_last_week >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_usage_last_month_check
    CHECK (usage_last_month >= 0);

ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_total_post_comments_check
    CHECK (total_post_comments >= 0);

-- Validate avg_post_score is non-negative
ALTER TABLE public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_avg_post_score_check
    CHECK (avg_post_score >= 0);

-- =============================================================================
-- ADDITIONAL COMPOSITE CONSTRAINTS
-- =============================================================================

-- Template key must be unique within (board_id, flair_type) combination if not null
-- Already handled by unique index in migration 1, but documented here

-- Post can only have one active flair (already enforced by unique index in migration 1)

-- User can only have one flair per board (already enforced by unique index in migration 1)

-- User can only have one filter config per board (already enforced by unique index in migration 2)

-- =============================================================================
-- VALIDATION FUNCTIONS (Optional advanced validations)
-- =============================================================================

-- Function to validate that custom text is only set when template allows editing
CREATE OR REPLACE FUNCTION public.validate_flair_custom_text() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    template_editable boolean;
    template_max_length integer;
BEGIN
    -- Get template settings
    SELECT is_editable, max_text_length INTO template_editable, template_max_length
    FROM flair_templates
    WHERE id = NEW.flair_template_id;

    -- If custom text is provided but template is not editable, raise error
    IF NEW.custom_text IS NOT NULL AND NOT template_editable THEN
        RAISE EXCEPTION 'Cannot set custom text for non-editable flair template';
    END IF;

    -- If custom text exceeds template's max length, raise error
    IF NEW.custom_text IS NOT NULL AND length(NEW.custom_text) > template_max_length THEN
        RAISE EXCEPTION 'Custom text exceeds maximum length of % characters', template_max_length;
    END IF;

    RETURN NEW;
END $$;

-- Apply custom text validation to post_flairs
CREATE TRIGGER validate_post_flair_custom_text
    BEFORE INSERT OR UPDATE ON public.post_flairs
    FOR EACH ROW
    WHEN (NEW.custom_text IS NOT NULL)
    EXECUTE FUNCTION public.validate_flair_custom_text();

-- Apply custom text validation to user_flairs
CREATE TRIGGER validate_user_flair_custom_text
    BEFORE INSERT OR UPDATE ON public.user_flairs
    FOR EACH ROW
    WHEN (NEW.custom_text IS NOT NULL)
    EXECUTE FUNCTION public.validate_flair_custom_text();

-- =============================================================================
-- VALIDATION FUNCTION: Ensure post flair uses 'post' type template
-- =============================================================================
CREATE OR REPLACE FUNCTION public.validate_post_flair_type() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    template_type character varying(10);
BEGIN
    -- Get template type
    SELECT flair_type INTO template_type
    FROM flair_templates
    WHERE id = NEW.flair_template_id;

    -- Ensure it's a post flair template
    IF template_type != 'post' THEN
        RAISE EXCEPTION 'Cannot assign user flair template to post';
    END IF;

    RETURN NEW;
END $$;

CREATE TRIGGER validate_post_flair_type_trigger
    BEFORE INSERT OR UPDATE ON public.post_flairs
    FOR EACH ROW
    EXECUTE FUNCTION public.validate_post_flair_type();

-- =============================================================================
-- VALIDATION FUNCTION: Ensure user flair uses 'user' type template
-- =============================================================================
CREATE OR REPLACE FUNCTION public.validate_user_flair_type() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    template_type character varying(10);
    template_board_id integer;
BEGIN
    -- Get template type and board
    SELECT flair_type, board_id INTO template_type, template_board_id
    FROM flair_templates
    WHERE id = NEW.flair_template_id;

    -- Ensure it's a user flair template
    IF template_type != 'user' THEN
        RAISE EXCEPTION 'Cannot assign post flair template to user';
    END IF;

    -- Ensure template belongs to the same board
    IF template_board_id != NEW.board_id THEN
        RAISE EXCEPTION 'Flair template does not belong to the specified board';
    END IF;

    RETURN NEW;
END $$;

CREATE TRIGGER validate_user_flair_type_trigger
    BEFORE INSERT OR UPDATE ON public.user_flairs
    FOR EACH ROW
    EXECUTE FUNCTION public.validate_user_flair_type();

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON FUNCTION public.validate_flair_custom_text() IS 'Validates that custom text is only set when template allows editing and respects max length';
COMMENT ON FUNCTION public.validate_post_flair_type() IS 'Ensures post flairs only use post-type templates';
COMMENT ON FUNCTION public.validate_user_flair_type() IS 'Ensures user flairs only use user-type templates from the same board';
