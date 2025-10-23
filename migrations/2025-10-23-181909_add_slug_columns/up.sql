-- Add slug columns to support SEO-friendly URLs
-- Following existing convention: VARCHAR with appropriate length, nullable initially

-- Add slug to posts table
ALTER TABLE posts
ADD COLUMN slug VARCHAR(80);

-- Add slug to comments table (for threaded discussions that might have URLs)
ALTER TABLE comments
ADD COLUMN slug VARCHAR(80);

-- Add slug to boards table (boards already have 'name' which serves as slug, but we'll add explicit slug field for future flexibility)
-- Note: Boards already use 'name' as URL identifier, so this is optional/future-proofing
-- ALTER TABLE boards ADD COLUMN slug VARCHAR(80);

-- Create indexes for efficient slug lookups
-- Single column indexes
CREATE INDEX idx_posts_slug ON posts(slug);
CREATE INDEX idx_comments_slug ON comments(slug);

-- Composite indexes for board-scoped uniqueness
CREATE INDEX idx_posts_board_slug ON posts(board_id, slug);
CREATE INDEX idx_comments_board_slug ON comments(board_id, slug);

-- Add comments for documentation
COMMENT ON COLUMN posts.slug IS 'URL-friendly slug generated from title for SEO-friendly URLs';
COMMENT ON COLUMN comments.slug IS 'URL-friendly slug for comment threads (optional, for future use)';
