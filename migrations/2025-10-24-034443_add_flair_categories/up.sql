-- Migration: Add Flair Categories
-- This migration adds flair categories to organize flair templates

-- =============================================================================
-- FLAIR CATEGORIES TABLE
-- =============================================================================
CREATE TABLE public.flair_categories (
    id integer NOT NULL,
    board_id integer NOT NULL,
    name character varying(100) NOT NULL,
    description text,
    color character varying(7), -- Hex color code for UI display
    display_order integer DEFAULT 0 NOT NULL,
    created_at timestamp without time zone DEFAULT now() NOT NULL,
    updated_at timestamp without time zone DEFAULT now() NOT NULL,
    created_by integer NOT NULL
);

-- Sequence for flair_categories
CREATE SEQUENCE public.flair_categories_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.flair_categories_id_seq OWNED BY public.flair_categories.id;
ALTER TABLE ONLY public.flair_categories ALTER COLUMN id SET DEFAULT nextval('public.flair_categories_id_seq'::regclass);

-- Primary key
ALTER TABLE ONLY public.flair_categories
    ADD CONSTRAINT flair_categories_pkey PRIMARY KEY (id);

-- Foreign keys
ALTER TABLE ONLY public.flair_categories
    ADD CONSTRAINT flair_categories_board_id_fkey
    FOREIGN KEY (board_id) REFERENCES public.boards(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.flair_categories
    ADD CONSTRAINT flair_categories_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES public.users(id) ON DELETE SET NULL;

-- Indexes
CREATE INDEX idx_flair_categories_board_id ON public.flair_categories(board_id);
CREATE INDEX idx_flair_categories_display_order ON public.flair_categories(board_id, display_order);

-- Unique constraint: board can't have duplicate category names
CREATE UNIQUE INDEX idx_flair_categories_unique_name
    ON public.flair_categories(board_id, name);

-- Comments
COMMENT ON TABLE public.flair_categories IS 'Categories for organizing flair templates within boards';
COMMENT ON COLUMN public.flair_categories.display_order IS 'Display order for UI presentation';

-- =============================================================================
-- ADD category_id TO flair_templates
-- =============================================================================
ALTER TABLE public.flair_templates
    ADD COLUMN category_id integer;

-- Foreign key
ALTER TABLE ONLY public.flair_templates
    ADD CONSTRAINT flair_templates_category_id_fkey
    FOREIGN KEY (category_id) REFERENCES public.flair_categories(id) ON DELETE SET NULL;

-- Index for category lookups
CREATE INDEX idx_flair_templates_category_id ON public.flair_templates(category_id);

COMMENT ON COLUMN public.flair_templates.category_id IS 'Optional category for organizing flairs';

-- =============================================================================
-- TRIGGER: Update updated_at timestamp
-- =============================================================================
CREATE OR REPLACE FUNCTION update_flair_category_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_flair_category_timestamp
    BEFORE UPDATE ON public.flair_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_flair_category_timestamp();
