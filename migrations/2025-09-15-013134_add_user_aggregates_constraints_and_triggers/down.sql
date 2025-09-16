-- Remove user_aggregates triggers
DROP TRIGGER IF EXISTS user_aggregates_comment_score ON comment_votes;
DROP TRIGGER IF EXISTS user_aggregates_comment_count ON comments;
DROP TRIGGER IF EXISTS user_aggregates_post_score ON post_votes;
DROP TRIGGER IF EXISTS user_aggregates_post_count ON posts;
DROP TRIGGER IF EXISTS user_aggregates_user ON users;

-- Remove user_aggregates functions
DROP FUNCTION IF EXISTS user_aggregates_comment_score();
DROP FUNCTION IF EXISTS user_aggregates_comment_count();
DROP FUNCTION IF EXISTS user_aggregates_post_score();
DROP FUNCTION IF EXISTS user_aggregates_post_count();
DROP FUNCTION IF EXISTS user_aggregates_user();

-- Remove foreign key constraint
ALTER TABLE user_aggregates DROP CONSTRAINT IF EXISTS user_aggregates_user_id_fkey;

-- Restore person_aggregates functions and triggers
CREATE OR REPLACE FUNCTION person_aggregates_person()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into person_aggregates (person_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from person_aggregates where person_id = OLD.id;
  END IF;
  return null;
end $function$;

CREATE OR REPLACE FUNCTION person_aggregates_post_count()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates
    set post_count = post_count + 1 where person_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates
    set post_count = post_count - 1 where person_id = OLD.creator_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    update person_aggregates ua
    set post_count = (
      select count(*)
      from posts p
      where p.creator_id = ua.person_id
      and p.is_deleted = false
      and p.is_removed = false
    )
    where ua.person_id = NEW.creator_id
      or ua.person_id = OLD.creator_id;
  END IF;
  return null;
end $function$;

CREATE OR REPLACE FUNCTION person_aggregates_post_score()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates ua
    set post_score = post_score + NEW.score where ua.person_id = (select creator_id from posts where id = NEW.post_id);
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates ua
    set post_score = post_score - OLD.score where ua.person_id = (select creator_id from posts where id = OLD.post_id);
  END IF;
  return null;
end $function$;

CREATE OR REPLACE FUNCTION person_aggregates_comment_count()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates
    set comment_count = comment_count + 1 where person_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates
    set comment_count = comment_count - 1 where person_id = OLD.creator_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    update person_aggregates ua
    set comment_count = (
      select count(*)
      from comments c
      where c.creator_id = ua.person_id
      and c.is_deleted = false
      and c.is_removed = false
    )
    where ua.person_id = NEW.creator_id
      or ua.person_id = OLD.creator_id;
  END IF;
  return null;
end $function$;

CREATE OR REPLACE FUNCTION person_aggregates_comment_score()
RETURNS trigger
LANGUAGE plpgsql
AS $function$
begin
  IF (TG_OP = 'INSERT') THEN
    update person_aggregates ua
    set comment_score = comment_score + NEW.score where ua.person_id = (select creator_id from comments where id = NEW.comment_id);
  ELSIF (TG_OP = 'DELETE') THEN
    update person_aggregates ua
    set comment_score = comment_score - OLD.score where ua.person_id = (select creator_id from comments where id = OLD.comment_id);
  END IF;
  return null;
end $function$;

-- Restore person_aggregates triggers
CREATE TRIGGER person_aggregates_person
  AFTER INSERT OR DELETE ON person
  FOR EACH ROW EXECUTE FUNCTION person_aggregates_person();

CREATE TRIGGER person_aggregates_post_count
  AFTER INSERT OR DELETE OR UPDATE ON posts
  FOR EACH ROW EXECUTE FUNCTION person_aggregates_post_count();

CREATE TRIGGER person_aggregates_post_score
  AFTER INSERT OR DELETE ON post_votes
  FOR EACH ROW EXECUTE FUNCTION person_aggregates_post_score();

CREATE TRIGGER person_aggregates_comment_count
  AFTER INSERT OR DELETE OR UPDATE ON comments
  FOR EACH ROW EXECUTE FUNCTION person_aggregates_comment_count();

CREATE TRIGGER person_aggregates_comment_score
  AFTER INSERT OR DELETE ON comment_votes
  FOR EACH ROW EXECUTE FUNCTION person_aggregates_comment_score();