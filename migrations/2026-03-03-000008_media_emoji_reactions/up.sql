-- Media, emoji, and reaction tables

-- ============================================================
-- uploads
-- ============================================================

CREATE TABLE uploads (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    original_name   TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    file_path       TEXT NOT NULL,
    upload_url      TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_uploads_user ON uploads (user_id);

-- ============================================================
-- content_uploads (join table linking uploads to posts/comments)
-- ============================================================

CREATE TABLE content_uploads (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upload_id   UUID NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
    post_id     UUID REFERENCES posts(id) ON DELETE CASCADE,
    comment_id  UUID REFERENCES comments(id) ON DELETE CASCADE,
    position    INT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_content_uploads_upload ON content_uploads (upload_id);
CREATE INDEX idx_content_uploads_post ON content_uploads (post_id) WHERE post_id IS NOT NULL;
CREATE INDEX idx_content_uploads_comment ON content_uploads (comment_id) WHERE comment_id IS NOT NULL;

-- ============================================================
-- emoji
-- ============================================================

CREATE TABLE emoji (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    shortcode       VARCHAR(128) NOT NULL,
    image_url       TEXT NOT NULL,
    alt_text        TEXT NOT NULL DEFAULT '',
    category        TEXT NOT NULL DEFAULT '',
    scope           emoji_scope NOT NULL DEFAULT 'global',
    board_id        UUID REFERENCES boards(id) ON DELETE CASCADE,
    created_by      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    usage_count     INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT emoji_shortcode_scope_unique UNIQUE (shortcode, board_id)
);

SELECT add_updated_at_trigger('emoji');

CREATE INDEX idx_emoji_shortcode ON emoji (shortcode);
CREATE INDEX idx_emoji_board ON emoji (board_id) WHERE board_id IS NOT NULL;
CREATE INDEX idx_emoji_active ON emoji (is_active) WHERE is_active = true;

-- ============================================================
-- emoji_keywords
-- ============================================================

CREATE TABLE emoji_keywords (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    emoji_id    UUID NOT NULL REFERENCES emoji(id) ON DELETE CASCADE,
    keyword     VARCHAR(128) NOT NULL
);

CREATE INDEX idx_emoji_keywords_emoji ON emoji_keywords (emoji_id);
CREATE INDEX idx_emoji_keywords_keyword ON emoji_keywords (keyword);

-- ============================================================
-- reactions
-- ============================================================

CREATE TABLE reactions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id     UUID REFERENCES posts(id) ON DELETE CASCADE,
    comment_id  UUID REFERENCES comments(id) ON DELETE CASCADE,
    emoji       VARCHAR(100) NOT NULL,
    score       INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT reactions_has_target CHECK (
        post_id IS NOT NULL OR comment_id IS NOT NULL
    )
);

CREATE INDEX idx_reactions_user_post ON reactions (user_id, post_id) WHERE post_id IS NOT NULL;
CREATE INDEX idx_reactions_user_comment ON reactions (user_id, comment_id) WHERE comment_id IS NOT NULL;
CREATE INDEX idx_reactions_post_emoji ON reactions (post_id, emoji) WHERE post_id IS NOT NULL;
CREATE INDEX idx_reactions_comment_emoji ON reactions (comment_id, emoji) WHERE comment_id IS NOT NULL;

-- ============================================================
-- reaction_aggregates
-- ============================================================

CREATE TABLE reaction_aggregates (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id     UUID REFERENCES posts(id) ON DELETE CASCADE,
    comment_id  UUID REFERENCES comments(id) ON DELETE CASCADE,
    emoji       VARCHAR(100) NOT NULL,
    count       INT NOT NULL DEFAULT 0,

    CONSTRAINT reaction_aggregates_has_target CHECK (
        post_id IS NOT NULL OR comment_id IS NOT NULL
    )
);

CREATE INDEX idx_reaction_aggregates_post ON reaction_aggregates (post_id, emoji)
    WHERE post_id IS NOT NULL;
CREATE INDEX idx_reaction_aggregates_comment ON reaction_aggregates (comment_id, emoji)
    WHERE comment_id IS NOT NULL;

-- ============================================================
-- board_reaction_settings
-- ============================================================

CREATE TABLE board_reaction_settings (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id            UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    emoji_weights       JSONB NOT NULL DEFAULT '{}',
    is_reactions_enabled BOOLEAN NOT NULL DEFAULT true,

    CONSTRAINT board_reaction_settings_board_unique UNIQUE (board_id)
);

-- ============================================================
-- Trigger: maintain reaction_aggregates
-- ============================================================

CREATE OR REPLACE FUNCTION trg_reaction_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO reaction_aggregates (post_id, comment_id, emoji, count)
        VALUES (NEW.post_id, NEW.comment_id, NEW.emoji, 1)
        ON CONFLICT DO NOTHING;

        -- Try updating if insert conflicted
        UPDATE reaction_aggregates
        SET count = count + 1
        WHERE emoji = NEW.emoji
          AND (post_id = NEW.post_id OR (post_id IS NULL AND NEW.post_id IS NULL))
          AND (comment_id = NEW.comment_id OR (comment_id IS NULL AND NEW.comment_id IS NULL));

    ELSIF TG_OP = 'DELETE' THEN
        UPDATE reaction_aggregates
        SET count = count - 1
        WHERE emoji = OLD.emoji
          AND (post_id = OLD.post_id OR (post_id IS NULL AND OLD.post_id IS NULL))
          AND (comment_id = OLD.comment_id OR (comment_id IS NULL AND OLD.comment_id IS NULL));

        -- Clean up zero-count rows
        DELETE FROM reaction_aggregates
        WHERE count <= 0
          AND emoji = OLD.emoji
          AND (post_id = OLD.post_id OR (post_id IS NULL AND OLD.post_id IS NULL))
          AND (comment_id = OLD.comment_id OR (comment_id IS NULL AND OLD.comment_id IS NULL));
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reaction_aggregates_on_reaction
    AFTER INSERT OR DELETE ON reactions
    FOR EACH ROW EXECUTE FUNCTION trg_reaction_aggregates();
