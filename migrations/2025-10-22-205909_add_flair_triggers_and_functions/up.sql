-- Migration 3/4: Flair Triggers and Functions
-- This migration creates database functions and triggers to maintain
-- flair usage statistics and keep aggregate tables synchronized

-- =============================================================================
-- TRIGGER FUNCTION: Update flair_templates.updated_at
-- =============================================================================
-- Reuse existing trigger_set_timestamp function or create if needed
-- This function updates the updated_at column on row modification

-- Apply updated_at trigger to flair_templates
CREATE TRIGGER set_updated_at_flair_templates
    BEFORE UPDATE ON public.flair_templates
    FOR EACH ROW
    EXECUTE FUNCTION public.trigger_set_timestamp();

-- Apply updated_at trigger to user_flair_filters
CREATE TRIGGER set_updated_at_user_flair_filters
    BEFORE UPDATE ON public.user_flair_filters
    FOR EACH ROW
    EXECUTE FUNCTION public.trigger_set_timestamp();

-- Apply updated_at trigger to flair_aggregates
CREATE TRIGGER set_updated_at_flair_aggregates
    BEFORE UPDATE ON public.flair_aggregates
    FOR EACH ROW
    EXECUTE FUNCTION public.trigger_set_timestamp();

-- =============================================================================
-- TRIGGER FUNCTION: Auto-create flair_aggregates for new templates
-- =============================================================================
CREATE OR REPLACE FUNCTION public.flair_aggregates_template() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Create aggregate record for new flair template
        INSERT INTO flair_aggregates (flair_template_id, created_at, updated_at)
        VALUES (NEW.id, now(), now());
    ELSIF (TG_OP = 'DELETE') THEN
        -- Delete aggregate record when template is deleted (CASCADE should handle this)
        DELETE FROM flair_aggregates WHERE flair_template_id = OLD.id;
    END IF;
    RETURN NULL;
END $$;

-- Trigger to create/delete flair aggregates
CREATE TRIGGER flair_aggregates_template_trigger
    AFTER INSERT OR DELETE ON public.flair_templates
    FOR EACH ROW
    EXECUTE FUNCTION public.flair_aggregates_template();

-- =============================================================================
-- TRIGGER FUNCTION: Update template usage count on flair assignment
-- =============================================================================
CREATE OR REPLACE FUNCTION public.update_flair_template_usage() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Increment usage count on flair_templates
        UPDATE flair_templates
        SET usage_count = usage_count + 1
        WHERE id = NEW.flair_template_id;

        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement usage count on flair_templates
        UPDATE flair_templates
        SET usage_count = GREATEST(usage_count - 1, 0)
        WHERE id = OLD.flair_template_id;

        RETURN OLD;
    END IF;
    RETURN NULL;
END $$;

-- Apply to post_flairs
CREATE TRIGGER update_flair_template_usage_post
    AFTER INSERT OR DELETE ON public.post_flairs
    FOR EACH ROW
    EXECUTE FUNCTION public.update_flair_template_usage();

-- Apply to user_flairs
CREATE TRIGGER update_flair_template_usage_user
    AFTER INSERT OR DELETE ON public.user_flairs
    FOR EACH ROW
    EXECUTE FUNCTION public.update_flair_template_usage();

-- =============================================================================
-- TRIGGER FUNCTION: Update flair_aggregates on post flair changes
-- =============================================================================
CREATE OR REPLACE FUNCTION public.update_flair_aggregates_post() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    post_score_val integer;
    post_comments_val integer;
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Get post metrics
        SELECT COALESCE(score, 0), COALESCE(comments, 0)
        INTO post_score_val, post_comments_val
        FROM post_aggregates
        WHERE post_id = NEW.post_id;

        -- Update aggregate statistics
        UPDATE flair_aggregates
        SET
            total_usage_count = total_usage_count + 1,
            post_usage_count = post_usage_count + 1,
            total_post_score = total_post_score + post_score_val,
            total_post_comments = total_post_comments + post_comments_val,
            avg_post_score = (total_post_score + post_score_val)::numeric / GREATEST(post_usage_count + 1, 1),
            last_used_at = now(),
            updated_at = now()
        WHERE flair_template_id = NEW.flair_template_id;

        RETURN NEW;

    ELSIF (TG_OP = 'DELETE') THEN
        -- Get post metrics
        SELECT COALESCE(score, 0), COALESCE(comments, 0)
        INTO post_score_val, post_comments_val
        FROM post_aggregates
        WHERE post_id = OLD.post_id;

        -- Update aggregate statistics
        UPDATE flair_aggregates
        SET
            total_usage_count = GREATEST(total_usage_count - 1, 0),
            post_usage_count = GREATEST(post_usage_count - 1, 0),
            total_post_score = GREATEST(total_post_score - post_score_val, 0),
            total_post_comments = GREATEST(total_post_comments - post_comments_val, 0),
            avg_post_score = CASE
                WHEN post_usage_count - 1 > 0 THEN (total_post_score - post_score_val)::numeric / (post_usage_count - 1)
                ELSE 0.0
            END,
            updated_at = now()
        WHERE flair_template_id = OLD.flair_template_id;

        RETURN OLD;

    ELSIF (TG_OP = 'UPDATE') THEN
        -- Handle flair template change (rare case)
        IF (OLD.flair_template_id != NEW.flair_template_id) THEN
            -- Get post metrics
            SELECT COALESCE(score, 0), COALESCE(comments, 0)
            INTO post_score_val, post_comments_val
            FROM post_aggregates
            WHERE post_id = NEW.post_id;

            -- Decrement old template stats
            UPDATE flair_aggregates
            SET
                total_usage_count = GREATEST(total_usage_count - 1, 0),
                post_usage_count = GREATEST(post_usage_count - 1, 0),
                total_post_score = GREATEST(total_post_score - post_score_val, 0),
                total_post_comments = GREATEST(total_post_comments - post_comments_val, 0),
                avg_post_score = CASE
                    WHEN post_usage_count - 1 > 0 THEN (total_post_score - post_score_val)::numeric / (post_usage_count - 1)
                    ELSE 0.0
                END,
                updated_at = now()
            WHERE flair_template_id = OLD.flair_template_id;

            -- Increment new template stats
            UPDATE flair_aggregates
            SET
                total_usage_count = total_usage_count + 1,
                post_usage_count = post_usage_count + 1,
                total_post_score = total_post_score + post_score_val,
                total_post_comments = total_post_comments + post_comments_val,
                avg_post_score = (total_post_score + post_score_val)::numeric / GREATEST(post_usage_count + 1, 1),
                last_used_at = now(),
                updated_at = now()
            WHERE flair_template_id = NEW.flair_template_id;
        END IF;

        RETURN NEW;
    END IF;

    RETURN NULL;
END $$;

-- Apply trigger to post_flairs
CREATE TRIGGER update_flair_aggregates_post_trigger
    AFTER INSERT OR DELETE OR UPDATE ON public.post_flairs
    FOR EACH ROW
    EXECUTE FUNCTION public.update_flair_aggregates_post();

-- =============================================================================
-- TRIGGER FUNCTION: Update flair_aggregates on user flair changes
-- =============================================================================
CREATE OR REPLACE FUNCTION public.update_flair_aggregates_user() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Update aggregate statistics
        UPDATE flair_aggregates
        SET
            total_usage_count = total_usage_count + 1,
            user_usage_count = user_usage_count + 1,
            active_user_count = active_user_count + 1,
            last_used_at = now(),
            updated_at = now()
        WHERE flair_template_id = NEW.flair_template_id;

        RETURN NEW;

    ELSIF (TG_OP = 'DELETE') THEN
        -- Update aggregate statistics
        UPDATE flair_aggregates
        SET
            total_usage_count = GREATEST(total_usage_count - 1, 0),
            user_usage_count = GREATEST(user_usage_count - 1, 0),
            active_user_count = GREATEST(active_user_count - 1, 0),
            updated_at = now()
        WHERE flair_template_id = OLD.flair_template_id;

        RETURN OLD;

    ELSIF (TG_OP = 'UPDATE') THEN
        -- Handle flair template change
        IF (OLD.flair_template_id != NEW.flair_template_id) THEN
            -- Decrement old template stats
            UPDATE flair_aggregates
            SET
                total_usage_count = GREATEST(total_usage_count - 1, 0),
                user_usage_count = GREATEST(user_usage_count - 1, 0),
                active_user_count = GREATEST(active_user_count - 1, 0),
                updated_at = now()
            WHERE flair_template_id = OLD.flair_template_id;

            -- Increment new template stats
            UPDATE flair_aggregates
            SET
                total_usage_count = total_usage_count + 1,
                user_usage_count = user_usage_count + 1,
                active_user_count = active_user_count + 1,
                last_used_at = now(),
                updated_at = now()
            WHERE flair_template_id = NEW.flair_template_id;
        END IF;

        RETURN NEW;
    END IF;

    RETURN NULL;
END $$;

-- Apply trigger to user_flairs
CREATE TRIGGER update_flair_aggregates_user_trigger
    AFTER INSERT OR DELETE OR UPDATE ON public.user_flairs
    FOR EACH ROW
    EXECUTE FUNCTION public.update_flair_aggregates_user();

-- =============================================================================
-- TRIGGER FUNCTION: Update flair_aggregates when post scores change
-- =============================================================================
-- This trigger updates flair aggregate metrics when post_aggregates changes
CREATE OR REPLACE FUNCTION public.update_flair_aggregates_on_post_score() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    template_id integer;
    old_score integer;
    new_score integer;
    old_comments integer;
    new_comments integer;
    usage_count integer;
BEGIN
    -- Get the flair template for this post (if any)
    SELECT flair_template_id INTO template_id
    FROM post_flairs
    WHERE post_id = NEW.post_id;

    -- Only proceed if post has a flair
    IF template_id IS NOT NULL THEN
        old_score := COALESCE(OLD.score, 0);
        new_score := COALESCE(NEW.score, 0);
        old_comments := COALESCE(OLD.comments, 0);
        new_comments := COALESCE(NEW.comments, 0);

        -- Update flair aggregates with new scores
        UPDATE flair_aggregates
        SET
            total_post_score = total_post_score - old_score + new_score,
            total_post_comments = total_post_comments - old_comments + new_comments,
            avg_post_score = CASE
                WHEN post_usage_count > 0 THEN (total_post_score - old_score + new_score)::numeric / post_usage_count
                ELSE 0.0
            END,
            updated_at = now()
        WHERE flair_template_id = template_id;
    END IF;

    RETURN NEW;
END $$;

-- Apply trigger to post_aggregates updates
CREATE TRIGGER update_flair_aggregates_on_post_score_trigger
    AFTER UPDATE OF score, comments ON public.post_aggregates
    FOR EACH ROW
    WHEN (OLD.score IS DISTINCT FROM NEW.score OR OLD.comments IS DISTINCT FROM NEW.comments)
    EXECUTE FUNCTION public.update_flair_aggregates_on_post_score();

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON FUNCTION public.flair_aggregates_template() IS 'Auto-creates flair_aggregates record when new template is created';
COMMENT ON FUNCTION public.update_flair_template_usage() IS 'Updates usage_count on flair_templates when flairs are assigned/removed';
COMMENT ON FUNCTION public.update_flair_aggregates_post() IS 'Updates flair_aggregates statistics when post flairs change';
COMMENT ON FUNCTION public.update_flair_aggregates_user() IS 'Updates flair_aggregates statistics when user flairs change';
COMMENT ON FUNCTION public.update_flair_aggregates_on_post_score() IS 'Updates flair_aggregates when post scores/comments change';
