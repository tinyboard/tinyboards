-- Revert slug columns migration

-- Remove indexes first
DROP INDEX IF EXISTS idx_comments_board_slug;
DROP INDEX IF EXISTS idx_posts_board_slug;
DROP INDEX IF EXISTS idx_comments_slug;
DROP INDEX IF EXISTS idx_posts_slug;

-- Remove columns
ALTER TABLE comments DROP COLUMN IF EXISTS slug;
ALTER TABLE posts DROP COLUMN IF EXISTS slug;
