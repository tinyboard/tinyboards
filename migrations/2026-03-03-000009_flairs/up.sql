-- Flair system tables

-- ============================================================
-- flair_categories
-- ============================================================

CREATE TABLE flair_categories (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id        UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    description     TEXT,
    color           VARCHAR(7),
    display_order   INT NOT NULL DEFAULT 0,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT add_updated_at_trigger('flair_categories');

CREATE INDEX idx_flair_categories_board ON flair_categories (board_id);

-- ============================================================
-- flair_templates
-- ============================================================

CREATE TABLE flair_templates (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id            UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    flair_type          flair_type NOT NULL,
    template_name       VARCHAR(100) NOT NULL,
    template_key        VARCHAR(50),
    text_display        VARCHAR(64) NOT NULL,
    text_color          VARCHAR(7) NOT NULL DEFAULT '#FFFFFF',
    background_color    VARCHAR(7) NOT NULL DEFAULT '#333333',
    style_config        JSONB NOT NULL DEFAULT '{}',
    emoji_ids           INTEGER[] NOT NULL DEFAULT '{}',
    is_mod_only         BOOLEAN NOT NULL DEFAULT false,
    is_editable         BOOLEAN NOT NULL DEFAULT false,
    max_emoji_count     INT NOT NULL DEFAULT 3,
    max_text_length     INT NOT NULL DEFAULT 64,
    is_requires_approval BOOLEAN NOT NULL DEFAULT false,
    display_order       INT NOT NULL DEFAULT 0,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    usage_count         INT NOT NULL DEFAULT 0,
    category_id         UUID REFERENCES flair_categories(id) ON DELETE SET NULL,
    created_by          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT add_updated_at_trigger('flair_templates');

CREATE INDEX idx_flair_templates_board ON flair_templates (board_id);
CREATE INDEX idx_flair_templates_board_type ON flair_templates (board_id, flair_type);
CREATE INDEX idx_flair_templates_board_active ON flair_templates (board_id, is_active)
    WHERE is_active = true;
CREATE INDEX idx_flair_templates_category ON flair_templates (category_id)
    WHERE category_id IS NOT NULL;
CREATE INDEX idx_flair_templates_style ON flair_templates USING GIN (style_config);
CREATE INDEX idx_flair_templates_emoji ON flair_templates USING GIN (emoji_ids);

-- ============================================================
-- post_flairs
-- ============================================================

CREATE TABLE post_flairs (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id                 UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    flair_template_id       UUID NOT NULL REFERENCES flair_templates(id) ON DELETE CASCADE,
    custom_text             VARCHAR(64),
    custom_text_color       VARCHAR(7),
    custom_background_color VARCHAR(7),
    assigned_by             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_original_author      BOOLEAN NOT NULL DEFAULT false,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_post_flairs_post ON post_flairs (post_id);
CREATE INDEX idx_post_flairs_template ON post_flairs (flair_template_id);

-- ============================================================
-- user_flairs
-- ============================================================

CREATE TABLE user_flairs (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    board_id                UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    flair_template_id       UUID NOT NULL REFERENCES flair_templates(id) ON DELETE CASCADE,
    custom_text             VARCHAR(64),
    custom_text_color       VARCHAR(7),
    custom_background_color VARCHAR(7),
    is_approved             BOOLEAN NOT NULL DEFAULT false,
    approved_at             TIMESTAMPTZ,
    approved_by             UUID REFERENCES users(id) ON DELETE SET NULL,
    is_self_assigned        BOOLEAN NOT NULL DEFAULT true,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT user_flairs_unique UNIQUE (user_id, board_id)
);

CREATE INDEX idx_user_flairs_board ON user_flairs (board_id);
CREATE INDEX idx_user_flairs_template ON user_flairs (flair_template_id);

-- ============================================================
-- user_flair_filters
-- ============================================================

CREATE TABLE user_flair_filters (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    board_id            UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    filter_mode         filter_mode NOT NULL DEFAULT 'include',
    included_flair_ids  INTEGER[] NOT NULL DEFAULT '{}',
    excluded_flair_ids  INTEGER[] NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT user_flair_filters_unique UNIQUE (user_id, board_id)
);

SELECT add_updated_at_trigger('user_flair_filters');

-- ============================================================
-- flair_aggregates
-- ============================================================

CREATE TABLE flair_aggregates (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flair_template_id   UUID NOT NULL REFERENCES flair_templates(id) ON DELETE CASCADE,
    total_usage_count   INT NOT NULL DEFAULT 0,
    post_usage_count    INT NOT NULL DEFAULT 0,
    user_usage_count    INT NOT NULL DEFAULT 0,
    active_user_count   INT NOT NULL DEFAULT 0,
    usage_last_day      INT NOT NULL DEFAULT 0,
    usage_last_week     INT NOT NULL DEFAULT 0,
    usage_last_month    INT NOT NULL DEFAULT 0,
    avg_post_score      NUMERIC NOT NULL DEFAULT 0,
    total_post_comments INT NOT NULL DEFAULT 0,
    total_post_score    INT NOT NULL DEFAULT 0,
    trending_score      NUMERIC NOT NULL DEFAULT 0,
    hot_rank            NUMERIC NOT NULL DEFAULT 0,
    last_used_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT flair_aggregates_template_unique UNIQUE (flair_template_id)
);

SELECT add_updated_at_trigger('flair_aggregates');

-- ============================================================
-- Flair triggers
-- ============================================================

-- Auto-create flair_aggregates on flair_template insert
CREATE OR REPLACE FUNCTION trg_flair_aggregates_on_template()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO flair_aggregates (flair_template_id) VALUES (NEW.id);
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM flair_aggregates WHERE flair_template_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER flair_aggregates_on_template
    AFTER INSERT OR DELETE ON flair_templates
    FOR EACH ROW EXECUTE FUNCTION trg_flair_aggregates_on_template();

-- Update usage counts when post_flairs change
CREATE OR REPLACE FUNCTION trg_flair_usage_post()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE flair_templates SET usage_count = usage_count + 1 WHERE id = NEW.flair_template_id;
        UPDATE flair_aggregates
        SET post_usage_count = post_usage_count + 1,
            total_usage_count = total_usage_count + 1,
            last_used_at = now()
        WHERE flair_template_id = NEW.flair_template_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE flair_templates SET usage_count = usage_count - 1 WHERE id = OLD.flair_template_id;
        UPDATE flair_aggregates
        SET post_usage_count = post_usage_count - 1,
            total_usage_count = total_usage_count - 1
        WHERE flair_template_id = OLD.flair_template_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER flair_usage_on_post_flair
    AFTER INSERT OR DELETE ON post_flairs
    FOR EACH ROW EXECUTE FUNCTION trg_flair_usage_post();

-- Update usage counts when user_flairs change
CREATE OR REPLACE FUNCTION trg_flair_usage_user()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE flair_aggregates
        SET user_usage_count = user_usage_count + 1,
            total_usage_count = total_usage_count + 1,
            last_used_at = now()
        WHERE flair_template_id = NEW.flair_template_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE flair_aggregates
        SET user_usage_count = user_usage_count - 1,
            total_usage_count = total_usage_count - 1
        WHERE flair_template_id = OLD.flair_template_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER flair_usage_on_user_flair
    AFTER INSERT OR DELETE ON user_flairs
    FOR EACH ROW EXECUTE FUNCTION trg_flair_usage_user();
