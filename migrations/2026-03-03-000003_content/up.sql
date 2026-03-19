-- Content tables: posts, comments, post_votes, comment_votes

-- ============================================================
-- languages (lookup table, INT PK for efficiency)
-- ============================================================

CREATE TABLE languages (
    id      SERIAL PRIMARY KEY,
    code    VARCHAR(3) NOT NULL,
    name    TEXT NOT NULL,
    CONSTRAINT languages_code_unique UNIQUE (code)
);

-- ============================================================
-- posts
-- ============================================================

CREATE TABLE posts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           VARCHAR(200) NOT NULL,
    post_type       post_type NOT NULL DEFAULT 'text',
    url             TEXT,
    thumbnail_url   TEXT,
    body            TEXT NOT NULL DEFAULT '',
    body_html       TEXT NOT NULL DEFAULT '',
    image           TEXT,
    alt_text        TEXT,
    slug            VARCHAR(80) NOT NULL DEFAULT '',

    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    board_id        UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    language_id     INT REFERENCES languages(id) ON DELETE SET NULL,

    -- Flags
    is_removed          BOOLEAN NOT NULL DEFAULT false,
    is_locked           BOOLEAN NOT NULL DEFAULT false,
    is_nsfw             BOOLEAN NOT NULL DEFAULT false,
    is_featured_board   BOOLEAN NOT NULL DEFAULT false,
    is_featured_local   BOOLEAN NOT NULL DEFAULT false,

    -- Approval
    approval_status approval_status NOT NULL DEFAULT 'approved',
    approved_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at     TIMESTAMPTZ,

    -- Embeds
    embed_title         TEXT,
    embed_description   TEXT,
    embed_video_url     TEXT,
    source_url          TEXT,
    last_crawl_date     TIMESTAMPTZ,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);

SELECT add_updated_at_trigger('posts');

CREATE INDEX idx_posts_creator ON posts (creator_id);
CREATE INDEX idx_posts_board ON posts (board_id);
CREATE INDEX idx_posts_created_at ON posts (created_at);
CREATE INDEX idx_posts_board_created ON posts (board_id, created_at);
CREATE INDEX idx_posts_slug ON posts (slug) WHERE slug != '';

-- ============================================================
-- comments
-- ============================================================

CREATE TABLE comments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    body            TEXT NOT NULL,
    body_html       TEXT NOT NULL,
    slug            VARCHAR(80) NOT NULL DEFAULT '',

    creator_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id         UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    parent_id       UUID REFERENCES comments(id) ON DELETE CASCADE,
    board_id        UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    language_id     INT REFERENCES languages(id) ON DELETE SET NULL,

    level           INT NOT NULL DEFAULT 0,

    -- Flags
    is_removed  BOOLEAN NOT NULL DEFAULT false,
    is_locked   BOOLEAN NOT NULL DEFAULT false,
    is_read     BOOLEAN NOT NULL DEFAULT false,
    is_pinned   BOOLEAN NOT NULL DEFAULT false,

    -- Approval
    approval_status approval_status NOT NULL DEFAULT 'approved',
    approved_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at     TIMESTAMPTZ,

    quoted_comment_id UUID REFERENCES comments(id) ON DELETE SET NULL,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at  TIMESTAMPTZ
);

SELECT add_updated_at_trigger('comments');

CREATE INDEX idx_comments_creator ON comments (creator_id);
CREATE INDEX idx_comments_post ON comments (post_id);
CREATE INDEX idx_comments_parent ON comments (parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_comments_board ON comments (board_id);
CREATE INDEX idx_comments_post_created ON comments (post_id, created_at);
CREATE INDEX idx_comments_slug ON comments (slug) WHERE slug != '';

-- ============================================================
-- post_votes
-- ============================================================

CREATE TABLE post_votes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id     UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    score       SMALLINT NOT NULL CHECK (score IN (-1, 1)),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT post_votes_user_post_unique UNIQUE (user_id, post_id)
);

CREATE INDEX idx_post_votes_post ON post_votes (post_id);
CREATE INDEX idx_post_votes_user ON post_votes (user_id);

-- ============================================================
-- comment_votes
-- ============================================================

CREATE TABLE comment_votes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    comment_id  UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    post_id     UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    score       SMALLINT NOT NULL CHECK (score IN (-1, 1)),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT comment_votes_user_comment_unique UNIQUE (user_id, comment_id)
);

CREATE INDEX idx_comment_votes_comment ON comment_votes (comment_id);
CREATE INDEX idx_comment_votes_user ON comment_votes (user_id);
