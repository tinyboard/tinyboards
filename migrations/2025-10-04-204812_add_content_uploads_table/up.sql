-- Create content_uploads junction table to link uploads to posts/comments
CREATE TABLE content_uploads (
    id SERIAL PRIMARY KEY,
    upload_id INT NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
    post_id INT REFERENCES posts(id) ON DELETE CASCADE,
    comment_id INT REFERENCES comments(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    position INT DEFAULT 0,
    CONSTRAINT content_uploads_check CHECK (
        (post_id IS NOT NULL AND comment_id IS NULL) OR
        (post_id IS NULL AND comment_id IS NOT NULL)
    )
);

-- Create indexes for performance
CREATE INDEX idx_content_uploads_post_id ON content_uploads(post_id);
CREATE INDEX idx_content_uploads_comment_id ON content_uploads(comment_id);
CREATE INDEX idx_content_uploads_upload_id ON content_uploads(upload_id);

-- Function to associate existing uploads with posts based on image/url fields
-- This migrates existing single-file posts to the new multi-file system
CREATE OR REPLACE FUNCTION migrate_existing_post_uploads() RETURNS void AS $$
DECLARE
    post_record RECORD;
    upload_record RECORD;
BEGIN
    -- Migrate posts with image field
    FOR post_record IN
        SELECT id, image, creator_id
        FROM posts
        WHERE image IS NOT NULL
    LOOP
        -- Try to find matching upload by URL
        SELECT * INTO upload_record
        FROM uploads
        WHERE upload_url = post_record.image
        LIMIT 1;

        IF FOUND THEN
            -- Create association if it doesn't already exist
            INSERT INTO content_uploads (upload_id, post_id, position)
            VALUES (upload_record.id, post_record.id, 0)
            ON CONFLICT DO NOTHING;
        END IF;
    END LOOP;

    -- Also check URL field for image links (if they're from our uploads)
    FOR post_record IN
        SELECT id, url, creator_id
        FROM posts
        WHERE url IS NOT NULL
        AND (url LIKE '%/media/%' OR url LIKE '%/emojis/%' OR url LIKE '%/avatars/%')
    LOOP
        -- Try to find matching upload by URL
        SELECT * INTO upload_record
        FROM uploads
        WHERE upload_url = post_record.url
        LIMIT 1;

        IF FOUND THEN
            -- Create association if it doesn't already exist
            INSERT INTO content_uploads (upload_id, post_id, position)
            VALUES (upload_record.id, post_record.id, 1)
            ON CONFLICT DO NOTHING;
        END IF;
    END LOOP;

    RAISE NOTICE 'Successfully migrated existing post uploads';
END;
$$ LANGUAGE plpgsql;

-- Run the migration function
SELECT migrate_existing_post_uploads();

-- Drop the function after migration (keep it clean)
DROP FUNCTION migrate_existing_post_uploads();
