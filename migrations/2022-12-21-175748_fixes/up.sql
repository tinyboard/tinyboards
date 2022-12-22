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
      coalesce(0, sum(pv.score)) as score
      -- User join because posts could be empty
      from users u 
      left join posts p on u.id = p.creator_id
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
      coalesce(0, sum(cv.score)) as score
      -- User join because comments could be empty
      from users u 
      left join comments c on u.id = c.creator_id
      left join comment_votes cv on c.id = cv.comment_id
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
