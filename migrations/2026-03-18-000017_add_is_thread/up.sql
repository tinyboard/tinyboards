-- Add is_thread column to posts table to distinguish thread posts from feed posts.
-- Previously the section type (thread vs feed) was passed during creation but never persisted,
-- so threads could not be filtered or identified after creation.
ALTER TABLE posts ADD COLUMN is_thread BOOLEAN NOT NULL DEFAULT FALSE;

-- Index for efficient filtering by section type within a board
CREATE INDEX idx_posts_board_is_thread ON posts (board_id, is_thread) WHERE deleted_at IS NULL AND is_removed = FALSE;
