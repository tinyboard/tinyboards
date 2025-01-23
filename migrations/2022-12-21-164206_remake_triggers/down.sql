-- This file should undo anything in `up.sql`
-- remake triggers to match new column/table names

-- drop dm triggers
drop trigger if exists refresh_dm on dms;
drop function refresh_dm;

-- recreate dm triggers
create or replace function refresh_private_message()
returns trigger language plpgsql
as $$
begin
  refresh materialized view concurrently private_message_mview;
  return null;
end $$;

create trigger refresh_private_message
after insert or update or delete or truncate
on dms
for each statement
execute procedure refresh_private_message();

-- drop initial user add trigger
drop trigger user_aggregates_user on users;
drop function user_aggregates_user;

-- recreate initial user add trigger
create function user_aggregates_user()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into user_aggregates (user_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from user_aggregates where user_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_user
after insert or delete on users
for each row
execute procedure user_aggregates_user();

-- drop post count
drop trigger if exists user_aggregates_post_count on posts;
drop function if exists user_aggregates_post_count;

-- remake post count
create function user_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set post_count = post_count + 1 where user_id = NEW.creator_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set post_count = post_count - 1 where user_id = OLD.creator_id;

    -- If the post gets deleted, the score calculation trigger won't fire, 
    -- so you need to re-calculate
    update user_aggregates ua
    set post_score = pd.score
    from (
      select u.id,
      coalesce(0, sum(pl.score)) as score
      -- User join because posts could be empty
      from user u 
      left join post p on u.id = p.creator_id
      left join post_votes pv on p.id = pv.post_id
      group by u.id
    ) pd 
    where ua.user_id = OLD.creator_id;

  END IF;
  return null;
end $$;

create trigger user_aggregates_post_count
after insert or delete on posts
for each row
execute procedure user_aggregates_post_count();

-- remake post score
drop trigger user_aggregates_post_score on post_votes;
drop function user_aggregates_post_score;

create function user_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set post_score = post_score + NEW.score
    from post p
    where ua.user_id = p.creator_id and p.id = NEW.post_id;
    
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set post_score = post_score - OLD.score
    from post p
    where ua.user_id = p.creator_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_post_score
after insert or delete on post_votes
for each row
execute procedure user_aggregates_post_score();

-- remake comment count
drop trigger user_aggregates_comment_count on comments;
drop function user_aggregates_comment_count;

create function user_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set comment_count = comment_count + 1 where user_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set comment_count = comment_count - 1 where user_id = OLD.creator_id;

    -- If the comment gets deleted, the score calculation trigger won't fire, 
    -- so you need to re-calculate
    update user_aggregates ua
    set comment_score = cd.score
    from (
      select u.id,
      coalesce(0, sum(cl.score)) as score
      -- User join because comments could be empty
      from user u 
      left join comment c on u.id = c.creator_id
      left join comment_vote cv on c.id = cv.comment_id
      group by u.id
    ) cd 
    where ua.user_id = OLD.creator_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_count
after insert or delete on comments
for each row
execute procedure user_aggregates_comment_count();

-- remake comment score
drop trigger user_aggregates_comment_score on comment_votes;
drop function user_aggregates_comment_score;

create function user_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set comment_score = comment_score + NEW.score
    from comment c
    where ua.user_id = c.creator_id and c.id = NEW.comment_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set comment_score = comment_score - OLD.score
    from comment c
    where ua.user_id = c.creator_id and c.id = OLD.comment_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_score
after insert or delete on comment_votes
for each row
execute procedure user_aggregates_comment_score();

-- POST AGGREGATES
-- remake comment count
drop trigger post_aggregates_comment_count on comments;
drop function post_aggregates_comment_count;

create function post_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set comments = comments + 1
    where pa.post_id = NEW.post_id;

    -- A 2 day necro-bump limit
    update post_aggregates pa
    set newest_comment_time = NEW.creation_date
    where pa.post_id = NEW.post_id
    and published > ('now'::timestamp - '2 days'::interval);
  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger post_aggregates_comment_count
after insert or delete on comments
for each row
execute procedure post_aggregates_comment_count();

-- remake post score
drop trigger post_aggregates_score on post_votes;
drop function post_aggregates_score;

create function post_aggregates_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set score = score + NEW.score,
    upvotes = case when NEW.score = 1 then upvotes + 1 else upvotes end,
    downvotes = case when NEW.score = -1 then downvotes + 1 else downvotes end
    where pa.post_id = NEW.post_id;

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set score = score - OLD.score,
    upvotes = case when OLD.score = 1 then upvotes - 1 else upvotes end,
    downvotes = case when OLD.score = -1 then downvotes - 1 else downvotes end
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;

  END IF;
  return null;
end $$;

create trigger post_aggregates_score
after insert or delete on post_votes
for each row
execute procedure post_aggregates_score();

-- remake post stickied
drop trigger post_aggregates_stickied on posts;
drop function post_aggregates_stickied;

create function post_aggregates_stickied()
returns trigger language plpgsql
as $$
begin
  update post_aggregates pa
  set stickied = NEW.is_stickied
  where pa.post_id = NEW.id;

  return null;
end $$;

create trigger post_aggregates_stickied
after update on posts
for each row
when (OLD.is_stickied is distinct from NEW.is_stickied)
execute procedure post_aggregates_stickied();

-- REMAKE BOARD AGGREGATES
-- remake post count
drop trigger board_aggregates_post_count on posts;
drop function board_aggregates_post_count;

create function board_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates 
    set posts = posts + 1 where board_id = NEW.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates 
    set posts = posts - 1 where board_id = OLD.board_id;

    -- Update the counts if the post got deleted
    update board_aggregates ba
    set posts = coalesce(bd.posts, 0),
    comments = coalesce(bd.comments, 0)
    from ( 
      select 
      b.id,
      count(distinct p.id) as posts,
      count(distinct ct.id) as comments
      from board b
      left join post p on b.id = p.board_id
      left join comment ct on p.id = ct.post_id
      group by b.id
    ) bd 
    where ba.board_id = OLD.board_id;
  END IF;
  return null;
end $$;

create trigger board_aggregates_post_count
after insert or delete on posts
for each row
execute procedure board_aggregates_post_count();

-- remake comment count
drop trigger board_aggregates_comment_count on comments;
drop function board_aggregates_comment_count;

create function board_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates ba
    set comments = comments + 1 from comments c, posts p
    where p.id = c.post_id 
    and p.id = NEW.post_id 
    and ba.board_id = p.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates ba
    set comments = comments - 1 from comments c, posts p
    where p.id = c.post_id 
    and p.id = OLD.post_id 
    and ba.board_id = p.board_id;

  END IF;
  return null;
end $$;

create trigger board_aggregates_comment_count
after insert or delete on comments
for each row
execute procedure board_aggregates_comment_count();

-- remake member count
drop trigger board_aggregates_subscriber_count on board_subscriptions;
drop function board_aggregates_subscriber_count;

create function board_aggregates_subscriber_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update board_aggregates 
    set subscribers = subscribers + 1 where board_id = NEW.board_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update board_aggregates 
    set subscribers = subscribers - 1 where board_id = OLD.board_id;
  END IF;
  return null;
end $$;

create trigger board_aggregates_subscriber_count
after insert or delete on board_subscriptions
for each row
execute procedure board_aggregates_subscriber_count();

-- COMMENT TRIGGERS
-- remake comment score
drop trigger comment_aggregates_score on comment_votes;
drop function comment_aggregates_score;

create function comment_aggregates_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update comment_aggregates ca
    set score = score + NEW.score,
    upvotes = case when NEW.score = 1 then upvotes + 1 else upvotes end,
    downvotes = case when NEW.score = -1 then downvotes + 1 else downvotes end
    where ca.comment_id = NEW.comment_id;

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to comment because that comment may not exist anymore
    update comment_aggregates ca
    set score = score - OLD.score,
    upvotes = case when OLD.score = 1 then upvotes - 1 else upvotes end,
    downvotes = case when OLD.score = -1 then downvotes - 1 else downvotes end
    from comment c
    where ca.comment_id = c.id
    and ca.comment_id = OLD.comment_id;

  END IF;
  return null;
end $$;

create trigger comment_aggregates_score
after insert or delete on comment_votes
for each row
execute procedure comment_aggregates_score();
