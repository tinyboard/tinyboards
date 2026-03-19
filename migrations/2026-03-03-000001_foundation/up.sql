-- Foundation: extensions, enum types, and utility functions

-- ============================================================
-- Extensions
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================
-- Enum types
-- ============================================================

CREATE TYPE registration_mode AS ENUM (
    'open',
    'invite_only',
    'application_required',
    'closed'
);

CREATE TYPE post_type AS ENUM (
    'text',
    'link',
    'image',
    'video'
);

CREATE TYPE sort_type AS ENUM (
    'hot',
    'new',
    'top',
    'old',
    'most_comments',
    'controversial'
);

CREATE TYPE listing_type AS ENUM (
    'all',
    'subscribed',
    'local'
);

CREATE TYPE approval_status AS ENUM (
    'pending',
    'approved',
    'rejected'
);

CREATE TYPE editor_mode AS ENUM (
    'richtext',
    'markdown',
    'plaintext'
);

CREATE TYPE notification_kind AS ENUM (
    'comment_reply',
    'post_reply',
    'mention',
    'private_message',
    'mod_action',
    'system'
);

CREATE TYPE moderation_action AS ENUM (
    'ban_user',
    'unban_user',
    'ban_from_board',
    'unban_from_board',
    'remove_post',
    'restore_post',
    'remove_comment',
    'restore_comment',
    'lock_post',
    'unlock_post',
    'lock_comment',
    'unlock_comment',
    'feature_post',
    'unfeature_post',
    'remove_board',
    'restore_board',
    'hide_board',
    'unhide_board',
    'add_mod',
    'remove_mod',
    'add_admin',
    'remove_admin',
    'purge_user',
    'purge_post',
    'purge_comment',
    'purge_board'
);

CREATE TYPE wiki_permission AS ENUM (
    'public',
    'members',
    'mods_only',
    'locked'
);

CREATE TYPE flair_type AS ENUM (
    'post',
    'user'
);

CREATE TYPE filter_mode AS ENUM (
    'include',
    'exclude'
);

CREATE TYPE emoji_scope AS ENUM (
    'global',
    'board'
);

CREATE TYPE report_status AS ENUM (
    'pending',
    'resolved',
    'dismissed'
);

-- ============================================================
-- Utility functions
-- ============================================================

-- Automatic updated_at trigger function
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Helper to attach the updated_at trigger to a table
CREATE OR REPLACE FUNCTION add_updated_at_trigger(table_name TEXT)
RETURNS VOID AS $$
BEGIN
    EXECUTE format(
        'CREATE TRIGGER set_updated_at BEFORE UPDATE ON %I
         FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
        table_name
    );
END;
$$ LANGUAGE plpgsql;

-- Hot rank algorithm: decays score over time
CREATE OR REPLACE FUNCTION hot_rank(score NUMERIC, published TIMESTAMPTZ)
RETURNS INTEGER AS $$
DECLARE
    hours_diff NUMERIC;
    result NUMERIC;
BEGIN
    hours_diff := EXTRACT(EPOCH FROM (now() - published)) / 3600.0;
    IF hours_diff <= 0 THEN
        hours_diff := 0.1;
    END IF;
    result := 10000.0 * log(greatest(abs(score), 1)) * sign(score) / power(hours_diff + 2, 1.8);
    RETURN GREATEST(CAST(result AS INTEGER), 0);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Controversy rank: higher when upvotes and downvotes are balanced and total is high
CREATE OR REPLACE FUNCTION controversy_rank(upvotes NUMERIC, downvotes NUMERIC)
RETURNS FLOAT8 AS $$
DECLARE
    total NUMERIC;
    smaller NUMERIC;
    larger NUMERIC;
BEGIN
    IF upvotes <= 0 OR downvotes <= 0 THEN
        RETURN 0;
    END IF;
    total := upvotes + downvotes;
    IF upvotes > downvotes THEN
        smaller := downvotes;
        larger := upvotes;
    ELSE
        smaller := upvotes;
        larger := downvotes;
    END IF;
    RETURN (total ^ 0.8) * (smaller::float8 / larger::float8);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- RLS helper: returns the current application user ID from session variable
CREATE OR REPLACE FUNCTION current_app_user_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_user_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql STABLE;
