-- Migration 2/4: Flair Filters and Aggregates
-- This migration creates supporting tables for flair filtering and statistics tracking

-- =============================================================================
-- USER FLAIR FILTERS TABLE
-- =============================================================================
-- Stores user preferences for filtering posts by flair within specific boards
CREATE TABLE public.user_flair_filters (
    id integer NOT NULL,
    user_id integer NOT NULL,
    board_id integer NOT NULL,

    -- Filter mode: 'include' (whitelist) or 'exclude' (blacklist)
    filter_mode character varying(10) DEFAULT 'exclude'::character varying NOT NULL,

    -- Arrays of flair template IDs to include/exclude
    included_flair_ids integer[] DEFAULT ARRAY[]::integer[] NOT NULL, -- Used when filter_mode = 'include'
    excluded_flair_ids integer[] DEFAULT ARRAY[]::integer[] NOT NULL, -- Used when filter_mode = 'exclude'

    -- Metadata
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);

-- Sequence for user_flair_filters
CREATE SEQUENCE public.user_flair_filters_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_flair_filters_id_seq OWNED BY public.user_flair_filters.id;
ALTER TABLE ONLY public.user_flair_filters ALTER COLUMN id SET DEFAULT nextval('public.user_flair_filters_id_seq'::regclass);

-- =============================================================================
-- FLAIR AGGREGATES TABLE
-- =============================================================================
-- Tracks usage statistics and trending metrics for flair templates
CREATE TABLE public.flair_aggregates (
    id integer NOT NULL,
    flair_template_id integer NOT NULL,

    -- Usage statistics
    total_usage_count integer DEFAULT 0 NOT NULL, -- Total times this flair has been used
    post_usage_count integer DEFAULT 0 NOT NULL, -- Times used on posts
    user_usage_count integer DEFAULT 0 NOT NULL, -- Times used as user flair
    active_user_count integer DEFAULT 0 NOT NULL, -- Current users with this flair

    -- Time-based usage tracking (for trending)
    usage_last_day integer DEFAULT 0 NOT NULL,
    usage_last_week integer DEFAULT 0 NOT NULL,
    usage_last_month integer DEFAULT 0 NOT NULL,

    -- Engagement metrics
    avg_post_score numeric(10,2) DEFAULT 0.0 NOT NULL, -- Average score of posts with this flair
    total_post_comments integer DEFAULT 0 NOT NULL, -- Total comments on posts with this flair
    total_post_score integer DEFAULT 0 NOT NULL, -- Total score of posts with this flair

    -- Trending calculation (updated periodically by background job)
    trending_score numeric(10,4) DEFAULT 0.0 NOT NULL,
    hot_rank numeric(10,4) DEFAULT 0.0 NOT NULL,

    -- Timestamps
    last_used_at timestamp without time zone, -- Last time flair was assigned
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL
);

-- Sequence for flair_aggregates
CREATE SEQUENCE public.flair_aggregates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.flair_aggregates_id_seq OWNED BY public.flair_aggregates.id;
ALTER TABLE ONLY public.flair_aggregates ALTER COLUMN id SET DEFAULT nextval('public.flair_aggregates_id_seq'::regclass);

-- =============================================================================
-- PRIMARY KEYS
-- =============================================================================
ALTER TABLE ONLY public.user_flair_filters
    ADD CONSTRAINT user_flair_filters_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_pkey PRIMARY KEY (id);

-- =============================================================================
-- FOREIGN KEY CONSTRAINTS
-- =============================================================================

-- User flair filters foreign keys
ALTER TABLE ONLY public.user_flair_filters
    ADD CONSTRAINT user_flair_filters_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_flair_filters
    ADD CONSTRAINT user_flair_filters_board_id_fkey
    FOREIGN KEY (board_id) REFERENCES public.boards(id) ON DELETE CASCADE;

-- Flair aggregates foreign keys
ALTER TABLE ONLY public.flair_aggregates
    ADD CONSTRAINT flair_aggregates_flair_template_id_fkey
    FOREIGN KEY (flair_template_id) REFERENCES public.flair_templates(id) ON DELETE CASCADE;

-- =============================================================================
-- INDEXES FOR PERFORMANCE
-- =============================================================================

-- User flair filters indexes
CREATE INDEX idx_user_flair_filters_user_id ON public.user_flair_filters(user_id);
CREATE INDEX idx_user_flair_filters_board_id ON public.user_flair_filters(board_id);
CREATE INDEX idx_user_flair_filters_user_board ON public.user_flair_filters(user_id, board_id);
CREATE INDEX idx_user_flair_filters_filter_mode ON public.user_flair_filters(filter_mode);
-- GIN indexes for array queries
CREATE INDEX idx_user_flair_filters_included_ids ON public.user_flair_filters USING gin(included_flair_ids);
CREATE INDEX idx_user_flair_filters_excluded_ids ON public.user_flair_filters USING gin(excluded_flair_ids);

-- Flair aggregates indexes
CREATE INDEX idx_flair_aggregates_flair_template_id ON public.flair_aggregates(flair_template_id);
CREATE INDEX idx_flair_aggregates_total_usage ON public.flair_aggregates(total_usage_count DESC);
CREATE INDEX idx_flair_aggregates_trending_score ON public.flair_aggregates(trending_score DESC);
CREATE INDEX idx_flair_aggregates_hot_rank ON public.flair_aggregates(hot_rank DESC);
CREATE INDEX idx_flair_aggregates_last_used ON public.flair_aggregates(last_used_at DESC NULLS LAST);
CREATE INDEX idx_flair_aggregates_avg_score ON public.flair_aggregates(avg_post_score DESC);

-- Composite indexes for common queries
CREATE INDEX idx_flair_aggregates_template_trending ON public.flair_aggregates(flair_template_id, trending_score DESC);
CREATE INDEX idx_flair_aggregates_template_usage ON public.flair_aggregates(flair_template_id, total_usage_count DESC);

-- =============================================================================
-- UNIQUE CONSTRAINTS
-- =============================================================================

-- A user can only have one filter configuration per board
CREATE UNIQUE INDEX idx_user_flair_filters_unique_user_board
    ON public.user_flair_filters(user_id, board_id);

-- Each flair template should have exactly one aggregate record
CREATE UNIQUE INDEX idx_flair_aggregates_unique_template
    ON public.flair_aggregates(flair_template_id);

-- =============================================================================
-- INITIALIZE AGGREGATES FOR EXISTING TEMPLATES
-- =============================================================================
-- Create aggregate records for any existing flair templates
INSERT INTO public.flair_aggregates (flair_template_id, created_at, updated_at)
SELECT id, now(), now()
FROM public.flair_templates
ON CONFLICT DO NOTHING;

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON TABLE public.user_flair_filters IS 'User preferences for filtering posts by flair within boards';
COMMENT ON TABLE public.flair_aggregates IS 'Usage statistics and trending metrics for flair templates';

COMMENT ON COLUMN public.user_flair_filters.filter_mode IS 'Filter mode: include (whitelist) or exclude (blacklist)';
COMMENT ON COLUMN public.user_flair_filters.included_flair_ids IS 'Array of flair template IDs to show (whitelist mode)';
COMMENT ON COLUMN public.user_flair_filters.excluded_flair_ids IS 'Array of flair template IDs to hide (blacklist mode)';

COMMENT ON COLUMN public.flair_aggregates.trending_score IS 'Calculated trending score based on recent usage and engagement';
COMMENT ON COLUMN public.flair_aggregates.hot_rank IS 'Hot ranking algorithm score for flair popularity';
COMMENT ON COLUMN public.flair_aggregates.avg_post_score IS 'Average score of posts tagged with this flair';
