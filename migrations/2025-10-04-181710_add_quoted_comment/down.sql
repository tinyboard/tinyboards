DROP INDEX IF EXISTS idx_comments_quoted_comment_id;
ALTER TABLE comments DROP CONSTRAINT IF EXISTS fk_quoted_comment;
ALTER TABLE comments DROP COLUMN IF EXISTS quoted_comment_id;
