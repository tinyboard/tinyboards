-- Add foreign key constraint for user_aggregates table
ALTER TABLE user_aggregates ADD CONSTRAINT user_aggregates_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Create user_aggregates functions to replace person_aggregates functions

-- Function for user creation/deletion
CREATE OR REPLACE FUNCTION user_aggregates_user()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into user_aggregates (user_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from user_aggregates where user_id = OLD.id;
  END IF;
  return null;
end $function$;

-- Function for post count updates
CREATE OR REPLACE FUNCTION user_aggregates_post_count()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates
    set post_count = post_count + 1 where user_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates
    set post_count = post_count - 1 where user_id = OLD.creator_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    update user_aggregates ua
    set post_count = (
      select count(*)
      from posts p
      where p.creator_id = ua.user_id
      and p.is_deleted = false
      and p.is_removed = false
    )
    where ua.user_id = NEW.creator_id
      or ua.user_id = OLD.creator_id;
  END IF;
  return null;
end $function$;

-- Function for post score updates
CREATE OR REPLACE FUNCTION user_aggregates_post_score()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates ua
    set post_score = post_score + NEW.score where ua.user_id = (select creator_id from posts where id = NEW.post_id);
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set post_score = post_score - OLD.score where ua.user_id = (select creator_id from posts where id = OLD.post_id);
  END IF;
  return null;
end $function$;

-- Function for comment count updates
CREATE OR REPLACE FUNCTION user_aggregates_comment_count()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates
    set comment_count = comment_count + 1 where user_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates
    set comment_count = comment_count - 1 where user_id = OLD.creator_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    update user_aggregates ua
    set comment_count = (
      select count(*)
      from comments c
      where c.creator_id = ua.user_id
      and c.is_deleted = false
      and c.is_removed = false
    )
    where ua.user_id = NEW.creator_id
      or ua.user_id = OLD.creator_id;
  END IF;
  return null;
end $function$;

-- Function for comment score updates
CREATE OR REPLACE FUNCTION user_aggregates_comment_score()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates ua
    set comment_score = comment_score + NEW.score where ua.user_id = (select creator_id from comments where id = NEW.comment_id);
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set comment_score = comment_score - OLD.score where ua.user_id = (select creator_id from comments where id = OLD.comment_id);
  END IF;
  return null;
end $function$;

-- Create triggers for user_aggregates

-- Trigger for user creation/deletion
CREATE TRIGGER user_aggregates_user
  AFTER INSERT OR DELETE ON users
  FOR EACH ROW EXECUTE FUNCTION user_aggregates_user();

-- Trigger for post count updates
CREATE TRIGGER user_aggregates_post_count
  AFTER INSERT OR DELETE OR UPDATE ON posts
  FOR EACH ROW EXECUTE FUNCTION user_aggregates_post_count();

-- Trigger for post score updates
CREATE TRIGGER user_aggregates_post_score
  AFTER INSERT OR DELETE ON post_votes
  FOR EACH ROW EXECUTE FUNCTION user_aggregates_post_score();

-- Trigger for comment count updates
CREATE TRIGGER user_aggregates_comment_count
  AFTER INSERT OR DELETE OR UPDATE ON comments
  FOR EACH ROW EXECUTE FUNCTION user_aggregates_comment_count();

-- Trigger for comment score updates
CREATE TRIGGER user_aggregates_comment_score
  AFTER INSERT OR DELETE ON comment_votes
  FOR EACH ROW EXECUTE FUNCTION user_aggregates_comment_score();

-- Remove old person_aggregates triggers
DROP TRIGGER IF EXISTS person_aggregates_person ON person;
DROP TRIGGER IF EXISTS person_aggregates_post_count ON posts;
DROP TRIGGER IF EXISTS person_aggregates_post_score ON post_votes;
DROP TRIGGER IF EXISTS person_aggregates_comment_count ON comments;
DROP TRIGGER IF EXISTS person_aggregates_comment_score ON comment_votes;

-- Remove old person_aggregates functions
DROP FUNCTION IF EXISTS person_aggregates_person();
DROP FUNCTION IF EXISTS person_aggregates_post_count();
DROP FUNCTION IF EXISTS person_aggregates_post_score();
DROP FUNCTION IF EXISTS person_aggregates_comment_count();
DROP FUNCTION IF EXISTS person_aggregates_comment_score();