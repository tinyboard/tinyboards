-- Wiki tables

-- ============================================================
-- wiki_pages
-- ============================================================

CREATE TABLE wiki_pages (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id        UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    slug            VARCHAR(100) NOT NULL,
    title           VARCHAR(200) NOT NULL,
    body            TEXT NOT NULL,
    body_html       TEXT NOT NULL,
    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_edited_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    view_permission wiki_permission NOT NULL DEFAULT 'public',
    edit_permission wiki_permission NOT NULL DEFAULT 'members',
    is_locked       BOOLEAN NOT NULL DEFAULT false,
    display_order   INT,
    parent_id       UUID REFERENCES wiki_pages(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT wiki_pages_board_slug_unique UNIQUE (board_id, slug)
);

SELECT add_updated_at_trigger('wiki_pages');

CREATE INDEX idx_wiki_pages_board ON wiki_pages (board_id);
CREATE INDEX idx_wiki_pages_parent ON wiki_pages (parent_id) WHERE parent_id IS NOT NULL;

-- ============================================================
-- wiki_page_revisions
-- ============================================================

CREATE TABLE wiki_page_revisions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id         UUID NOT NULL REFERENCES wiki_pages(id) ON DELETE CASCADE,
    revision_number INT NOT NULL,
    editor_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    edit_summary    TEXT,
    body            TEXT NOT NULL,
    body_html       TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_wiki_revisions_page ON wiki_page_revisions (page_id);
CREATE INDEX idx_wiki_revisions_page_number ON wiki_page_revisions (page_id, revision_number DESC);

-- ============================================================
-- wiki_approved_contributors
-- ============================================================

CREATE TABLE wiki_approved_contributors (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id    UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_by    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT wiki_contributors_unique UNIQUE (board_id, user_id)
);

-- ============================================================
-- Trigger: create initial revision when wiki page is inserted
-- ============================================================

CREATE OR REPLACE FUNCTION trg_wiki_page_initial_revision()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO wiki_page_revisions (page_id, revision_number, editor_id, edit_summary, body, body_html)
    VALUES (NEW.id, 1, NEW.creator_id, 'Initial version', NEW.body, NEW.body_html);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER wiki_page_initial_revision
    AFTER INSERT ON wiki_pages
    FOR EACH ROW EXECUTE FUNCTION trg_wiki_page_initial_revision();
