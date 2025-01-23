-- drop the aggregate tables, triggers, and functions here
drop table user_aggregates;
drop trigger user_aggregates_post_score on post_like;
drop trigger user_aggregates_comment_score on comment_like;
drop function user_aggregates_post_score, user_aggregates_comment_score;

drop table post_aggregates;
drop trigger post_aggregates_score on post_like;
drop function post_aggregates_score;

drop table comment_aggregates;
drop trigger comment_aggregates_score on comment_like;
drop function comment_aggregates_score;

-- dropping indexes
drop index idx_post_like_post;
drop index idx_post_like_user;

drop index idx_comment_like_comment;
drop index idx_comment_like_user;


-- rename the tables here
alter table post_like rename to post_vote;
alter table comment_like rename to comment_vote;

-- recreate the indexes
create index idx_post_vote_post on post_vote (post_id);
create index idx_post_vote_user on post_vote (user_id);

create index idx_comment_vote_comment on comment_vote (comment_id);
create index idx_comment_vote_user on comment_vote (user_id);

-----------------------------------------------------------------------------------------------------------------------------------------------------------
-- recreate user_aggregates
create table user_aggregates (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  post_count bigint not null default 0,
  post_score bigint not null default 0,
  comment_count bigint not null default 0,
  comment_score bigint not null default 0,
  unique (user_id)
);

insert into user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
  select u.id,
  coalesce(pd.posts, 0),
  coalesce(pd.score, 0),
  coalesce(cd.comments, 0),
  coalesce(cd.score, 0)
  from user_ u
  left join (
    select p.creator_id,
      count(distinct p.id) as posts,
      sum(pv.score) as score
      from post p
      left join post_vote pv on p.id = pv.post_id
      group by p.creator_id
    ) pd on u.id = pd.creator_id
  left join ( 
    select c.creator_id,
    count(distinct c.id) as comments,
    sum(cv.score) as score
    from comment c
    left join comment_vote cv on c.id = cv.comment_id
    group by c.creator_id
  ) cd on u.id = cd.creator_id;

-- Add user aggregate triggers 
-- post score
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
after insert or delete on post_vote
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
after insert or delete on comment_vote
for each row
execute procedure user_aggregates_comment_score();

-----------------------------------------------------------------------------------------------------------------------------------------------------------
-- recreate post_aggregates
create table post_aggregates (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  comments bigint not null default 0,
  score bigint not null default 0,
  upvotes bigint not null default 0,
  downvotes bigint not null default 0,
  stickied boolean not null default false,
  published timestamp not null default now(),
  newest_comment_time timestamp not null default now(),
  unique (post_id)
);

insert into post_aggregates (post_id, comments, score, upvotes, downvotes, stickied, published, newest_comment_time)
  select 
    p.id,
    coalesce(ct.comments, 0::bigint) as comments,
    coalesce(pv.score, 0::bigint) as score,
    coalesce(pv.upvotes, 0::bigint) as upvotes,
    coalesce(pv.downvotes, 0::bigint) as downvotes,
    p.stickied,
    p.published,
    greatest(ct.recent_comment_time, p.published) as newest_activity_time
  from post p
  left join ( 
    select comment.post_id,
    count(*) as comments,
    max(comment.published) as recent_comment_time
    from comment
    group by comment.post_id
  ) ct on ct.post_id = p.id
  left join ( 
    select post_vote.post_id,
    sum(post_vote.score) as score,
    sum(post_vote.score) filter (where post_vote.score = 1) as upvotes,
    -sum(post_vote.score) filter (where post_vote.score = '-1'::integer) as downvotes
    from post_vote
    group by post_vote.post_id
  ) pv on pv.post_id = p.id;

-- post score
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
after insert or delete on post_vote
for each row
execute procedure post_aggregates_score();
-----------------------------------------------------------------------------------------------------------------------------------------------------------
-- recreate comment_aggregates
create table comment_aggregates (
  id serial primary key,
  comment_id int references comment on update cascade on delete cascade not null,
  score bigint not null default 0,
  upvotes bigint not null default 0,
  downvotes bigint not null default 0,
  published timestamp not null default now(),
  unique (comment_id)
);

insert into comment_aggregates (comment_id, score, upvotes, downvotes, published)
  select 
    c.id,
    COALESCE(cv.total, 0::bigint) AS score,
    COALESCE(cv.up, 0::bigint) AS upvotes,
    COALESCE(cv.down, 0::bigint) AS downvotes,
    c.published
  from comment c
  left join ( select v.comment_id as id,
    sum(v.score) as total,
    count(
      case
      when v.score = 1 then 1
      else null::integer
      end) as up,
    count(
      case
      when v.score = '-1'::integer then 1
      else null::integer
      end) as down
    from comment_vote v
    group by v.comment_id) cv on cv.id = c.id;

-- Add comment aggregate triggers
-- comment score
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
after insert or delete on comment_vote
for each row
execute procedure comment_aggregates_score();
-----------------------------------------------------------------------------------------------------------------------------------------------------------

-- recreate views
drop view if exists post_view cascade;

create view post_view as
with all_post as
(
    select
    p.*,
    (select u.banned from user_ u where p.creator_id = u.id) as banned,
    (select bub.id::boolean from board_user_ban bub where p.creator_id = bub.user_id and p.board_id = bub.board_id) as banned_from_board,
    (select name from user_ where p.creator_id = user_.id) as creator_name,
    (select avatar from user_ where p.creator_id = user_.id) as creator_avatar,
    (select name from board where p.board_id = board.id) as board_name,
    (select removed from board b where p.board_id = b.id) as board_removed,
    (select deleted from board b where p.board_id = b.id) as board_deleted,
    (select nsfw from board b where p.board_id = b.id) as board_nsfw,
    (select count(*) from comment where comment.post_id = p.id) as number_of_comments,
    coalesce(sum(pv.score), 0) as score,
    count (case when pv.score = 1 then 1 else null end) as upvotes,
    count (case when pv.score = -1 then 1 else null end) as downvotes,
    hot_rank(coalesce(sum(pv.score), 0), p.published) as hot_rank
    from post p
    left join post_vote pv on p.id = pv.post_id
    group by p.id
)

select 
ap.*,
u.id as user_id,
coalesce(pv.score, 0) as my_vote,
(select bs.id::boolean from board_subscriber bs where u.id = bs.user_id and bs.board_id = ap.board_id) as subscribed,
(select pr.id::boolean from post_read pr where u.id = pr.user_id and pr.post_id = ap.id) as read,
(select ps.id::boolean from post_saved ps where u.id = ps.user_id and ps.post_id = ap.id) as saved
from user_ u
cross join all_post ap
left join post_vote pv on u.id = pv.user_id and ap.id = pv.post_id

union all 

select
ap.*,
null as user_id,
null as my_vote,
null as subscribed,
null as read,
null as saved
from all_post ap
;

-- user_view
drop view if exists user_view cascade;

create view user_view as 
select id,
name,
avatar,
email,
preferred_name,
admin,
banned,
email_notifications_enabled,
published,
(select count(*) from post p where p.creator_id = u.id) as number_of_posts,
(select coalesce(sum(score), 0) from post p, post_vote pv where u.id = p.creator_id and p.id = pv.post_id) as post_score,
(select count(*) from comment c where c.creator_id = u.id) as number_of_comments,
(select coalesce(sum(score), 0) from comment c, comment_vote cv where u.id = c.creator_id and c.id = cv.comment_id) as comment_score
from user_ u;

-- comment_view
drop view if exists comment_view cascade;

create view comment_view as
with all_comment as 
(
    select
    c.*,
    (select board_id from post p where p.id = c.post_id),
    (select u.banned from user_ u where c.creator_id = u.id) as banned,
    (select bub.id::boolean from board_user_ban bub, post p where c.creator_id = bub.user_id and p.id = c.post_id and p.board_id = bub.board_id) as user_banned_from_board,
    (select name from user_ where c.creator_id = user_.id) as creator_name,
    (select avatar from user_ where c.creator_id = user_.id) as creator_avatar,
    coalesce(sum(cv.score), 0) as score,
    count (case when cv.score = 1 then 1 else null end) as upvotes,
    count (case when cv.score = -1 then 1 else null end) as downvotes
    from comment c 
    left join comment_vote cv on c.id = cv.comment_id
    group by c.id
)

select
ac.*,
u.id as user_id,
coalesce(cv.score, 0) as my_vote,
(select cs.id::boolean from comment_saved cs where u.id = cs.user_id and cs.comment_id = ac.id) as saved
from user_ u 
cross join all_comment ac
left join comment_vote cv on u.id = cv.user_id and ac.id = cv.comment_id

union all 

select
    ac.*,
    null as user_id,
    null as my_vote,
    null as saved
from all_comment ac
;