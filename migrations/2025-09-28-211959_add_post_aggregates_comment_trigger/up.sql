-- Create function to update post aggregates comment count when comments are added/removed
CREATE OR REPLACE FUNCTION public.post_aggregates_comment_count() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates
    set comments = comments + 1,
        newest_comment_time = NEW.creation_date
    where post_id = NEW.post_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update post_aggregates
    set comments = comments - 1,
        newest_comment_time = (
          select max(creation_date)
          from comments
          where post_id = OLD.post_id
          and is_deleted = false
          and is_removed = false
        )
    where post_id = OLD.post_id;
  END IF;
  return null;
end $$;

-- Create trigger to update post aggregates when comments are inserted/deleted
CREATE TRIGGER post_aggregates_comment_count_trigger
    AFTER INSERT OR DELETE ON public.comments
    FOR EACH ROW
    EXECUTE FUNCTION public.post_aggregates_comment_count();
