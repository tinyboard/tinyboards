-- Backfill slugs for existing posts and comments
-- This migration generates slugs from titles/body and handles collisions

-- Create a helper function to generate URL-friendly slugs
CREATE OR REPLACE FUNCTION generate_slug(input_text TEXT, max_length INT DEFAULT 60)
RETURNS VARCHAR AS $$
DECLARE
    slug VARCHAR;
BEGIN
    -- Convert to lowercase
    slug := LOWER(input_text);

    -- Replace special characters and spaces with hyphens
    slug := regexp_replace(slug, '[^a-z0-9]+', '-', 'g');

    -- Remove leading/trailing hyphens
    slug := regexp_replace(slug, '^-+|-+$', '', 'g');

    -- Truncate to max length
    IF LENGTH(slug) > max_length THEN
        slug := SUBSTRING(slug FROM 1 FOR max_length);
        -- Remove trailing hyphens after truncation
        slug := regexp_replace(slug, '-+$', '', 'g');
    END IF;

    -- If slug is empty after processing, use a default
    IF slug = '' OR slug IS NULL THEN
        slug := 'post';
    END IF;

    RETURN slug;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Create a function to ensure unique slugs within board context
CREATE OR REPLACE FUNCTION ensure_unique_slug(
    base_slug VARCHAR,
    board_id_param INT,
    post_id_param INT
)
RETURNS VARCHAR AS $$
DECLARE
    final_slug VARCHAR;
    counter INT := 2;
    exists_check INT;
BEGIN
    final_slug := base_slug;

    -- Check if base slug is already taken by another post in same board
    SELECT COUNT(*) INTO exists_check
    FROM posts
    WHERE slug = final_slug
      AND board_id = board_id_param
      AND id != post_id_param;

    -- If taken, append numbers until we find a unique one
    WHILE exists_check > 0 LOOP
        final_slug := base_slug || '-' || counter;
        counter := counter + 1;

        SELECT COUNT(*) INTO exists_check
        FROM posts
        WHERE slug = final_slug
          AND board_id = board_id_param
          AND id != post_id_param;
    END LOOP;

    RETURN final_slug;
END;
$$ LANGUAGE plpgsql;

-- Backfill posts with slugs
-- Process in batches using a temporary table to track progress
DO $$
DECLARE
    post_record RECORD;
    base_slug VARCHAR;
    final_slug VARCHAR;
BEGIN
    FOR post_record IN
        SELECT id, title, board_id
        FROM posts
        WHERE slug IS NULL
        ORDER BY id
    LOOP
        -- Generate base slug from title
        base_slug := generate_slug(post_record.title, 60);

        -- Ensure uniqueness within board
        final_slug := ensure_unique_slug(base_slug, post_record.board_id, post_record.id);

        -- Update the post
        UPDATE posts
        SET slug = final_slug
        WHERE id = post_record.id;
    END LOOP;
END $$;

-- Backfill comments with slugs (using first 60 chars of body)
-- Comments don't necessarily need slugs for URLs, but we'll add them for consistency
DO $$
DECLARE
    comment_record RECORD;
    base_slug VARCHAR;
    final_slug VARCHAR;
    counter INT;
    exists_check INT;
BEGIN
    FOR comment_record IN
        SELECT id, body, board_id
        FROM comments
        WHERE slug IS NULL
        ORDER BY id
    LOOP
        -- Generate base slug from body (first 60 chars)
        base_slug := generate_slug(
            SUBSTRING(comment_record.body FROM 1 FOR 60),
            60
        );

        -- For comments, we'll just append the ID to keep it simple
        -- since we don't typically need human-readable comment URLs
        final_slug := base_slug || '-' || comment_record.id;

        -- Truncate if needed
        IF LENGTH(final_slug) > 80 THEN
            final_slug := SUBSTRING(final_slug FROM 1 FOR 80);
        END IF;

        -- Update the comment
        UPDATE comments
        SET slug = final_slug
        WHERE id = comment_record.id;
    END LOOP;
END $$;

-- Add NOT NULL constraint after backfilling
ALTER TABLE posts ALTER COLUMN slug SET NOT NULL;
ALTER TABLE comments ALTER COLUMN slug SET NOT NULL;

-- Add unique constraint for board-scoped slug uniqueness on posts
-- This prevents duplicate slugs within the same board
CREATE UNIQUE INDEX idx_posts_board_slug_unique ON posts(board_id, slug);

COMMENT ON FUNCTION generate_slug IS 'Generates URL-friendly slugs from text input';
COMMENT ON FUNCTION ensure_unique_slug IS 'Ensures slug uniqueness within board context by appending numbers if needed';
