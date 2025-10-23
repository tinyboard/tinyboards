-- Revert backfill migration

-- Remove unique constraint
DROP INDEX IF EXISTS idx_posts_board_slug_unique;

-- Remove NOT NULL constraints
ALTER TABLE comments ALTER COLUMN slug DROP NOT NULL;
ALTER TABLE posts ALTER COLUMN slug DROP NOT NULL;

-- Drop helper functions
DROP FUNCTION IF EXISTS ensure_unique_slug(VARCHAR, INT, INT);
DROP FUNCTION IF EXISTS generate_slug(TEXT, INT);

-- Clear all slug values
UPDATE comments SET slug = NULL;
UPDATE posts SET slug = NULL;
