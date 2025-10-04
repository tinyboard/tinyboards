-- Drop indexes
DROP INDEX IF EXISTS idx_content_uploads_upload_id;
DROP INDEX IF EXISTS idx_content_uploads_comment_id;
DROP INDEX IF EXISTS idx_content_uploads_post_id;

-- Drop table
DROP TABLE IF EXISTS content_uploads;
