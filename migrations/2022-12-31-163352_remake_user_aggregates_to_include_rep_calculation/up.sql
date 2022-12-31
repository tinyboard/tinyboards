-- drop the aggregate tables, triggers, and functions here
drop table user_aggregates;
drop trigger user_aggregates_post_score on post_votes;
drop trigger user_aggregates_comment_score on comment_votes;
drop function user_aggregates_post_score, user_aggregates_comment_score;


-- recreate user_aggregates
create table user_aggregates (
  id serial primary key,
  user_id int references users on update cascade on delete cascade not null,
  post_count bigint not null default 0,
  post_score bigint not null default 0,
  comment_count bigint not null default 0,
  comment_score bigint not null default 0,
  rep bigint not null default 0,
  unique (user_id)
);

insert into user_aggregates (user_id, post_count, post_score, comment_count, comment_score, rep)
  select u.id,
  coalesce(pd.posts, 0),
  coalesce(pd.score, 0),
  coalesce(cd.comments, 0),
  coalesce(cd.score, 0),
  round((coalesce(pd.score, 0) + coalesce(cd.score, 0)) / coalesce(pd.posts, 1)) -- coalesced 1 into posts to prevent division by zero
  from users u
  left join (
    select p.creator_id,
      count(distinct p.id) as posts,
      sum(pv.score) as score
      from posts p
      left join post_votes pv on p.id = pv.post_id
      group by p.creator_id
    ) pd on u.id = pd.creator_id
  left join ( 
    select c.creator_id,
    count(distinct c.id) as comments,
    sum(cv.score) as score
    from comments c
    left join comment_votes cv on c.id = cv.comment_id
    group by c.creator_id
  ) cd on u.id = cd.creator_id;

-- Add user aggregate triggers 
-- rep calculation
create function user_aggregates_rep()
returns trigger language plpgsql
as $$
begin
    IF (TG_OP = 'INSERT') THEN
        update user_aggregates ua
        set rep = calc.rep
        from (
            select
                u.id as user_id, 
                round((coalesce(pd.score, 0) + coalesce(cd.score, 0)) / coalesce(pd.posts, 1)) as rep 
            from users u
            left join (
                select p.creator_id,
                    count(distinct p.id) as posts,
                    sum(pv.score) as score
                    from posts p
                    left join post_votes pv on p.id = pv.post_id
                    group by p.creator_id
                ) pd on u.id = pd.creator_id
            left join (
                select c.creator_id,
                    count(distinct c.id) as comments,
                    sum(cv.score) as score
                    from comments c
                    left join comment_votes cv on c.id = cv.comment_id
                    group by c.creator_id
                ) cd on u.id = cd.creator_id
            ) calc 
        where ua.user_id = calc.user_id;
    ELSIF (TG_OP = 'DELETE') THEN
        update user_aggregates ua
        set rep = calc.rep
        from (
            select
                u.id as user_id, 
                round((coalesce(pd.score, 0) + coalesce(cd.score, 0)) / coalesce(pd.posts, 1)) as rep 
            from users u
            left join (
                select p.creator_id,
                    count(distinct p.id) as posts,
                    sum(pv.score) as score
                    from posts p
                    left join post_votes pv on p.id = pv.post_id
                    group by p.creator_id
                ) pd on u.id = pd.creator_id
            left join (
                select c.creator_id,
                    count(distinct c.id) as comments,
                    sum(cv.score) as score
                    from comments c
                    left join comment_votes cv on c.id = cv.comment_id
                    group by c.creator_id
                ) cd on u.id = cd.creator_id
            ) calc 
        where ua.user_id = calc.user_id;
    END IF;
    return null;
end $$;

-- post score
create function user_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the votercomment_votes
    update user_aggregates ua
    set post_score = post_score + NEW.score
    from posts p
    where ua.user_id = p.creator_id and p.id = NEW.post_id;
    
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set post_score = post_score - OLD.score
    from posts p
    where ua.user_id = p.creator_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_post_score
after insert or delete on post_votes
for each row
execute procedure user_aggregates_post_score();

-- comment score
create function user_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set comment_score = comment_score + NEW.score
    from comments c
    where ua.user_id = c.creator_id and c.id = NEW.comment_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set comment_score = comment_score - OLD.score
    from comments c
    where ua.user_id = c.creator_id and c.id = OLD.comment_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_score
after insert or delete on comment_votes
for each row
execute procedure user_aggregates_comment_score();