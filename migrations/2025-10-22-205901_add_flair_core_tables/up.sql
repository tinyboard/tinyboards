-- Migration 1/4: Core Flair Tables
-- This migration creates the foundational flair system tables:
-- flair_templates, post_flairs, and user_flairs with all necessary fields,
-- indexes, and foreign key relationships.

-- =============================================================================
-- FLAIR TEMPLATES TABLE
-- =============================================================================
-- Stores reusable flair templates for boards (both post and user flairs)
CREATE TABLE public.flair_templates (
    id integer NOT NULL,
    board_id integer NOT NULL,
    flair_type character varying(10) NOT NULL, -- 'post' or 'user'
    template_name character varying(100) NOT NULL,
    template_key character varying(50), -- Optional unique key for programmatic access
    text_display character varying(64) NOT NULL,

    -- Color configuration
    text_color character varying(7) DEFAULT '#000000'::character varying NOT NULL, -- Hex color code
    background_color character varying(7) DEFAULT '#FFFFFF'::character varying NOT NULL, -- Hex color code

    -- Advanced styling (JSONB for flexibility)
    style_config jsonb DEFAULT '{}'::jsonb NOT NULL,
    -- Example style_config structure:
    -- {
    --   "border_color": "#CCCCCC",
    --   "border_width": "1px",
    --   "border_radius": "4px",
    --   "font_weight": "normal",
    --   "font_size": "12px",
    --   "padding": "2px 8px",
    --   "icon_position": "left"
    -- }

    -- Emoji support (array of emoji IDs or unicode)
    emoji_ids integer[] DEFAULT ARRAY[]::integer[] NOT NULL,

    -- Permissions and configuration
    mod_only boolean DEFAULT false NOT NULL, -- Only mods can assign this flair
    is_editable boolean DEFAULT false NOT NULL, -- Users can customize text
    max_text_length integer DEFAULT 64 NOT NULL, -- Max length if editable
    requires_approval boolean DEFAULT false NOT NULL, -- User flair requires mod approval

    -- Display settings
    display_order integer DEFAULT 0 NOT NULL, -- Order in flair selection UI
    is_active boolean DEFAULT true NOT NULL, -- Can be assigned to new content

    -- Metadata
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL,
    created_by integer NOT NULL, -- User ID who created the template
    usage_count integer DEFAULT 0 NOT NULL -- Tracked by triggers
);

-- Sequence for flair_templates
CREATE SEQUENCE public.flair_templates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.flair_templates_id_seq OWNED BY public.flair_templates.id;
ALTER TABLE ONLY public.flair_templates ALTER COLUMN id SET DEFAULT nextval('public.flair_templates_id_seq'::regclass);

-- =============================================================================
-- POST FLAIRS TABLE
-- =============================================================================
-- Junction table linking posts to flair templates with optional customization
CREATE TABLE public.post_flairs (
    id integer NOT NULL,
    post_id integer NOT NULL,
    flair_template_id integer NOT NULL,

    -- Optional custom overrides (if template allows editing)
    custom_text character varying(64), -- Overrides template text if set
    custom_text_color character varying(7), -- Overrides template text color if set
    custom_background_color character varying(7), -- Overrides template bg color if set

    -- Metadata
    assigned_at timestamp without time zone DEFAULT now() NOT NULL,
    assigned_by integer NOT NULL, -- User ID who assigned the flair
    is_original_author boolean DEFAULT false NOT NULL -- True if assigned by post creator
);

-- Sequence for post_flairs
CREATE SEQUENCE public.post_flairs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_flairs_id_seq OWNED BY public.post_flairs.id;
ALTER TABLE ONLY public.post_flairs ALTER COLUMN id SET DEFAULT nextval('public.post_flairs_id_seq'::regclass);

-- =============================================================================
-- USER FLAIRS TABLE
-- =============================================================================
-- Junction table linking users to flair templates within board context
CREATE TABLE public.user_flairs (
    id integer NOT NULL,
    user_id integer NOT NULL,
    board_id integer NOT NULL,
    flair_template_id integer NOT NULL,

    -- Optional custom overrides (if template allows editing)
    custom_text character varying(64), -- Overrides template text if set
    custom_text_color character varying(7), -- Overrides template text color if set
    custom_background_color character varying(7), -- Overrides template bg color if set

    -- Approval workflow (for flairs requiring approval)
    is_approved boolean DEFAULT false NOT NULL,
    approved_at timestamp without time zone,
    approved_by integer, -- Moderator user ID

    -- Metadata
    assigned_at timestamp without time zone DEFAULT now() NOT NULL,
    is_self_assigned boolean DEFAULT true NOT NULL -- False if assigned by moderator
);

-- Sequence for user_flairs
CREATE SEQUENCE public.user_flairs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_flairs_id_seq OWNED BY public.user_flairs.id;
ALTER TABLE ONLY public.user_flairs ALTER COLUMN id SET DEFAULT nextval('public.user_flairs_id_seq'::regclass);

-- =============================================================================
-- PRIMARY KEYS
-- =============================================================================
ALTER TABLE ONLY public.flair_templates
    ADD CONSTRAINT flair_templates_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.post_flairs
    ADD CONSTRAINT post_flairs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.user_flairs
    ADD CONSTRAINT user_flairs_pkey PRIMARY KEY (id);

-- =============================================================================
-- FOREIGN KEY CONSTRAINTS
-- =============================================================================

-- Flair templates foreign keys
ALTER TABLE ONLY public.flair_templates
    ADD CONSTRAINT flair_templates_board_id_fkey
    FOREIGN KEY (board_id) REFERENCES public.boards(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.flair_templates
    ADD CONSTRAINT flair_templates_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;

-- Post flairs foreign keys
ALTER TABLE ONLY public.post_flairs
    ADD CONSTRAINT post_flairs_post_id_fkey
    FOREIGN KEY (post_id) REFERENCES public.posts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.post_flairs
    ADD CONSTRAINT post_flairs_flair_template_id_fkey
    FOREIGN KEY (flair_template_id) REFERENCES public.flair_templates(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.post_flairs
    ADD CONSTRAINT post_flairs_assigned_by_fkey
    FOREIGN KEY (assigned_by) REFERENCES public.users(id) ON DELETE SET NULL;

-- User flairs foreign keys
ALTER TABLE ONLY public.user_flairs
    ADD CONSTRAINT user_flairs_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_flairs
    ADD CONSTRAINT user_flairs_board_id_fkey
    FOREIGN KEY (board_id) REFERENCES public.boards(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_flairs
    ADD CONSTRAINT user_flairs_flair_template_id_fkey
    FOREIGN KEY (flair_template_id) REFERENCES public.flair_templates(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_flairs
    ADD CONSTRAINT user_flairs_approved_by_fkey
    FOREIGN KEY (approved_by) REFERENCES public.users(id) ON DELETE SET NULL;

-- =============================================================================
-- INDEXES FOR PERFORMANCE
-- =============================================================================

-- Flair templates indexes
CREATE INDEX idx_flair_templates_board_id ON public.flair_templates(board_id);
CREATE INDEX idx_flair_templates_flair_type ON public.flair_templates(flair_type);
CREATE INDEX idx_flair_templates_board_type ON public.flair_templates(board_id, flair_type);
CREATE INDEX idx_flair_templates_is_active ON public.flair_templates(is_active);
CREATE INDEX idx_flair_templates_template_key ON public.flair_templates(template_key) WHERE template_key IS NOT NULL;
CREATE INDEX idx_flair_templates_display_order ON public.flair_templates(board_id, display_order);
-- GIN index for JSONB style_config queries
CREATE INDEX idx_flair_templates_style_config ON public.flair_templates USING gin(style_config);
-- GIN index for array emoji_ids queries
CREATE INDEX idx_flair_templates_emoji_ids ON public.flair_templates USING gin(emoji_ids);

-- Post flairs indexes
CREATE INDEX idx_post_flairs_post_id ON public.post_flairs(post_id);
CREATE INDEX idx_post_flairs_flair_template_id ON public.post_flairs(flair_template_id);
CREATE INDEX idx_post_flairs_assigned_by ON public.post_flairs(assigned_by);
CREATE INDEX idx_post_flairs_assigned_at ON public.post_flairs(assigned_at);

-- User flairs indexes
CREATE INDEX idx_user_flairs_user_id ON public.user_flairs(user_id);
CREATE INDEX idx_user_flairs_board_id ON public.user_flairs(board_id);
CREATE INDEX idx_user_flairs_user_board ON public.user_flairs(user_id, board_id);
CREATE INDEX idx_user_flairs_flair_template_id ON public.user_flairs(flair_template_id);
CREATE INDEX idx_user_flairs_is_approved ON public.user_flairs(is_approved) WHERE is_approved = false;
CREATE INDEX idx_user_flairs_approved_by ON public.user_flairs(approved_by) WHERE approved_by IS NOT NULL;

-- =============================================================================
-- UNIQUE CONSTRAINTS
-- =============================================================================

-- A board can't have two templates with the same key (within same board and type)
CREATE UNIQUE INDEX idx_flair_templates_unique_key
    ON public.flair_templates(board_id, flair_type, template_key)
    WHERE template_key IS NOT NULL;

-- A post can only have one flair at a time
CREATE UNIQUE INDEX idx_post_flairs_unique_post
    ON public.post_flairs(post_id);

-- A user can only have one flair per board
CREATE UNIQUE INDEX idx_user_flairs_unique_user_board
    ON public.user_flairs(user_id, board_id);

-- =============================================================================
-- COMMENTS
-- =============================================================================

COMMENT ON TABLE public.flair_templates IS 'Reusable flair templates for boards (post and user flairs)';
COMMENT ON TABLE public.post_flairs IS 'Links posts to flair templates with optional customization';
COMMENT ON TABLE public.user_flairs IS 'Links users to flair templates within board context';

COMMENT ON COLUMN public.flair_templates.flair_type IS 'Type of flair: post or user';
COMMENT ON COLUMN public.flair_templates.template_key IS 'Optional unique identifier for programmatic access';
COMMENT ON COLUMN public.flair_templates.style_config IS 'JSONB object containing advanced CSS styling options';
COMMENT ON COLUMN public.flair_templates.emoji_ids IS 'Array of emoji IDs or unicode characters to display with flair';
COMMENT ON COLUMN public.flair_templates.mod_only IS 'If true, only moderators can assign this flair';
COMMENT ON COLUMN public.flair_templates.is_editable IS 'If true, users can customize the flair text';
COMMENT ON COLUMN public.flair_templates.requires_approval IS 'If true, user flair assignments require moderator approval';

COMMENT ON COLUMN public.user_flairs.is_approved IS 'Approval status for flairs requiring moderator approval';
COMMENT ON COLUMN public.user_flairs.is_self_assigned IS 'True if user assigned their own flair, false if assigned by moderator';
