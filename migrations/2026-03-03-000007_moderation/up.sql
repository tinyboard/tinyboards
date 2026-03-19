-- Moderation tables: unified log + reports

-- ============================================================
-- moderation_log (replaces all legacy mod_* and admin_purge_* tables)
-- ============================================================

CREATE TABLE moderation_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    moderator_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action_type     moderation_action NOT NULL,
    target_type     VARCHAR(20) NOT NULL,
    target_id       UUID NOT NULL,
    board_id        UUID REFERENCES boards(id) ON DELETE SET NULL,
    reason          TEXT,
    metadata        JSONB,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_moderation_log_moderator ON moderation_log (moderator_id);
CREATE INDEX idx_moderation_log_board ON moderation_log (board_id) WHERE board_id IS NOT NULL;
CREATE INDEX idx_moderation_log_action ON moderation_log (action_type);
CREATE INDEX idx_moderation_log_target ON moderation_log (target_type, target_id);
CREATE INDEX idx_moderation_log_created ON moderation_log (created_at DESC);

-- ============================================================
-- post_reports
-- ============================================================

CREATE TABLE post_reports (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id             UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    original_post_title VARCHAR(200) NOT NULL,
    original_post_url   TEXT,
    original_post_body  TEXT,
    reason              TEXT NOT NULL,
    status              report_status NOT NULL DEFAULT 'pending',
    resolver_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT add_updated_at_trigger('post_reports');

CREATE INDEX idx_post_reports_post ON post_reports (post_id);
CREATE INDEX idx_post_reports_status ON post_reports (status) WHERE status = 'pending';

-- ============================================================
-- comment_reports
-- ============================================================

CREATE TABLE comment_reports (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    creator_id              UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    comment_id              UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    original_comment_text   TEXT NOT NULL,
    reason                  TEXT NOT NULL,
    status                  report_status NOT NULL DEFAULT 'pending',
    resolver_id             UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT add_updated_at_trigger('comment_reports');

CREATE INDEX idx_comment_reports_comment ON comment_reports (comment_id);
CREATE INDEX idx_comment_reports_status ON comment_reports (status) WHERE status = 'pending';
