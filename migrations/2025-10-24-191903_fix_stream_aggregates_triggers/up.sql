-- Fix all trigger functions to use 'updated' instead of 'updated_at'
-- and 'creation_date' instead of 'created_at'
-- This includes stream aggregates and flair aggregates triggers

-- =============================================================================
-- STREAM AGGREGATES TRIGGERS
-- =============================================================================

-- Trigger function: Update flair subscription count
CREATE OR REPLACE FUNCTION stream_aggregates_flair_subscription()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE stream_aggregates
        SET
            flair_subscription_count = flair_subscription_count + 1,
            total_subscription_count = total_subscription_count + 1,
            updated = NOW()
        WHERE stream_id = NEW.stream_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE stream_aggregates
        SET
            flair_subscription_count = GREATEST(0, flair_subscription_count - 1),
            total_subscription_count = GREATEST(0, total_subscription_count - 1),
            updated = NOW()
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger function: Update board subscription count
CREATE OR REPLACE FUNCTION stream_aggregates_board_subscription()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE stream_aggregates
        SET
            board_subscription_count = board_subscription_count + 1,
            total_subscription_count = total_subscription_count + 1,
            updated = NOW()
        WHERE stream_id = NEW.stream_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE stream_aggregates
        SET
            board_subscription_count = GREATEST(0, board_subscription_count - 1),
            total_subscription_count = GREATEST(0, total_subscription_count - 1),
            updated = NOW()
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger function: Update follower count
CREATE OR REPLACE FUNCTION stream_aggregates_follower()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE stream_aggregates
        SET
            follower_count = follower_count + 1,
            updated = NOW()
        WHERE stream_id = NEW.stream_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE stream_aggregates
        SET
            follower_count = GREATEST(0, follower_count - 1),
            updated = NOW()
        WHERE stream_id = OLD.stream_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to recalculate all stream aggregates (for data repairs or updates)
CREATE OR REPLACE FUNCTION recalculate_stream_aggregates(p_stream_id INTEGER DEFAULT NULL)
RETURNS VOID AS $$
BEGIN
    IF p_stream_id IS NOT NULL THEN
        -- Recalculate for specific stream
        UPDATE stream_aggregates sa
        SET
            flair_subscription_count = (
                SELECT COUNT(*) FROM stream_flair_subscriptions
                WHERE stream_id = p_stream_id
            ),
            board_subscription_count = (
                SELECT COUNT(*) FROM stream_board_subscriptions
                WHERE stream_id = p_stream_id
            ),
            total_subscription_count = (
                SELECT COUNT(*) FROM stream_flair_subscriptions
                WHERE stream_id = p_stream_id
            ) + (
                SELECT COUNT(*) FROM stream_board_subscriptions
                WHERE stream_id = p_stream_id
            ),
            follower_count = (
                SELECT COUNT(*) FROM stream_followers
                WHERE stream_id = p_stream_id
            ),
            updated = NOW()
        WHERE sa.stream_id = p_stream_id;
    ELSE
        -- Recalculate for all streams
        UPDATE stream_aggregates sa
        SET
            flair_subscription_count = COALESCE((
                SELECT COUNT(*) FROM stream_flair_subscriptions sfs
                WHERE sfs.stream_id = sa.stream_id
            ), 0),
            board_subscription_count = COALESCE((
                SELECT COUNT(*) FROM stream_board_subscriptions sbs
                WHERE sbs.stream_id = sa.stream_id
            ), 0),
            total_subscription_count = COALESCE((
                SELECT COUNT(*) FROM stream_flair_subscriptions sfs
                WHERE sfs.stream_id = sa.stream_id
            ), 0) + COALESCE((
                SELECT COUNT(*) FROM stream_board_subscriptions sbs
                WHERE sbs.stream_id = sa.stream_id
            ), 0),
            follower_count = COALESCE((
                SELECT COUNT(*) FROM stream_followers sf
                WHERE sf.stream_id = sa.stream_id
            ), 0),
            updated = NOW();
    END IF;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- FLAIR AGGREGATES TRIGGERS
-- =============================================================================

-- Auto-create flair_aggregates for new templates
CREATE OR REPLACE FUNCTION public.flair_aggregates_template() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Create aggregate record for new flair template (use creation_date and updated)
        INSERT INTO flair_aggregates (flair_template_id, creation_date, updated)
        VALUES (NEW.id, now(), now());
    ELSIF (TG_OP = 'DELETE') THEN
        -- Delete aggregate record when template is deleted (CASCADE should handle this)
        DELETE FROM flair_aggregates WHERE flair_template_id = OLD.id;
    END IF;
    RETURN NULL;
END $$;

-- Update flair_aggregates on post flair changes
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
            updated = now()
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
            updated = now()
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
                updated = now()
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
                updated = now()
            WHERE flair_template_id = NEW.flair_template_id;
        END IF;

        RETURN NEW;
    END IF;

    RETURN NULL;
END $$;

-- Update flair_aggregates on user flair changes
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
            updated = now()
        WHERE flair_template_id = NEW.flair_template_id;

        RETURN NEW;

    ELSIF (TG_OP = 'DELETE') THEN
        -- Update aggregate statistics
        UPDATE flair_aggregates
        SET
            total_usage_count = GREATEST(total_usage_count - 1, 0),
            user_usage_count = GREATEST(user_usage_count - 1, 0),
            active_user_count = GREATEST(active_user_count - 1, 0),
            updated = now()
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
                updated = now()
            WHERE flair_template_id = OLD.flair_template_id;

            -- Increment new template stats
            UPDATE flair_aggregates
            SET
                total_usage_count = total_usage_count + 1,
                user_usage_count = user_usage_count + 1,
                active_user_count = active_user_count + 1,
                last_used_at = now(),
                updated = now()
            WHERE flair_template_id = NEW.flair_template_id;
        END IF;

        RETURN NEW;
    END IF;

    RETURN NULL;
END $$;

-- Update flair_aggregates when post scores change
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
            updated = now()
        WHERE flair_template_id = template_id;
    END IF;

    RETURN NEW;
END $$;
