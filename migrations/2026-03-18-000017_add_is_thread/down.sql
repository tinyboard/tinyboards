DROP INDEX IF EXISTS idx_posts_board_is_thread;
ALTER TABLE posts DROP COLUMN IF EXISTS is_thread;
