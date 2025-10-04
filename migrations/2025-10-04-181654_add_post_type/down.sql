DROP INDEX IF EXISTS idx_posts_board_post_type_created;
DROP INDEX IF EXISTS idx_posts_post_type;
ALTER TABLE posts DROP COLUMN IF EXISTS post_type;
