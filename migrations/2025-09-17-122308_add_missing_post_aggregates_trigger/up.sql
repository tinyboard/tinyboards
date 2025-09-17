-- Add missing trigger to create post_aggregates entries when posts are created
-- This trigger was missing from the remake_triggers migration

-- Drop existing trigger if it exists
DROP TRIGGER IF EXISTS post_aggregates_post ON posts;

-- Create or replace the function
CREATE OR REPLACE FUNCTION post_aggregates_post()
RETURNS trigger LANGUAGE plpgsql
AS $$
BEGIN
  IF (TG_OP = 'INSERT') THEN
    INSERT INTO post_aggregates (post_id, creation_date, newest_comment_time)
    VALUES (NEW.id, NEW.creation_date, NEW.creation_date);
  ELSIF (TG_OP = 'DELETE') THEN
    DELETE FROM post_aggregates WHERE post_id = OLD.id;
  END IF;
  RETURN null;
END $$;

-- Create the trigger
CREATE TRIGGER post_aggregates_post
AFTER INSERT OR DELETE ON posts
FOR EACH ROW
EXECUTE PROCEDURE post_aggregates_post();
