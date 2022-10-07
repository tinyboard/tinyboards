-- Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity
create or replace function hot_rank(
  score numeric,
  published timestamp without time zone)
returns integer as $$
begin
  -- hours_diff:=EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600
  return floor(10000*log(greatest(1,score+3)) / power(((EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600) + 2), 1.8))::integer;
end; $$
LANGUAGE plpgsql;

create view post_view as
with all_post as
(
    select
    p.*,
    (select name from user_ where p.creator_id = user_.id) as creator_name,
    (select name from board where p.board_id = board.id) as board_name,
    (select removed from board b where p.board_id = b.id) as board_removed,
    (select count(*) from comment where comment.post_id = p.id) as number_of_comments,
    coalesce(sum(pl.score), 0) as score,
    count (case when pl.score = 1 then 1 else null end) as upvotes,
    count (case when pl.score = -1 then 1 else null end) as downvotes,
    hot_rank(coalesce(sum(pl.score), 0), p.published) as hot_rank
    from post p
    left join post_like pl on p.id = pl.post_id
    group by p.id
)

select 
ap.*,
u.id as user_id,
coalesce(pl.score, 0) as my_vote,
(select bs.id::boolean from board_subscriber bs where u.id = bs.user_id and bs.board_id = ap.board_id) as subscribed,
(select pr.id::boolean from post_read pr where u.id = pr.user_id and pr.post_id = ap.id) as read,
(select ps.id::boolean from post_saved ps where u.id = ps.user_id and ps.post_id = ap.id) as saved
from user_ u
cross join all_post ap
left join post_like pl on u.id = pl.user_id and ap.id = pl.post_id

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


-- board view
create view board_view as
with all_board as
(
    select *,
    (select name from user_ u where b.creator_id = u.id) as creator_name,
    (select name from tag t where b.tag_id = t.id) as board_tag,
    (select count(*) from board_subscriber bs where bs.board_id = b.id) as number_of_subscribers,
    (select count(*) from post p where p.board_id = b.id) as number_of_posts,
    (select count(*) from comment c, post p where b.id = p.board_id and p.id = c.post_id) as number_of_comments
    from board b
)

select
ab.*,
u.id as user_id,
(select bs.id::boolean from board_subscriber bs where u.id = bs.user_id and ab.id = bs.board_id) as subscribed
from user_ u 
cross join all_board ab

union all

select 
ab.*,
null as user_id,
null as subscribed
from all_board ab
;

-- board_moderator_view
create view board_moderator_view as
select *,
(select name from user_ u where bm.user_id = u.id) as user_name,
(select name from board b where bm.board_id = b.id) as board_name
from board_moderator bm;

-- board_subscriber view
create view board_subscriber_view as 
select *,
(select name from user_ u where bs.user_id = u.id) as user_name,
(select name from board b where bs.board_id = b.id) as board_name
from board_subscriber bs;

-- board_user_ban_view
create view board_user_ban_view as 
select *,
(select name from user_ u where bub.user_id = u.id) as user_name,
(select name from board b where bub.board_id = b.id) as board_name
from board_user_ban bub;

-- site view
create view site_view as 
select *,
(select name from user_ u where s.creator_id = u.id) as creator_name,
(select count(*) from user_) as number_of_users,
(select count(*) from post) as number_of_posts,
(select count(*) from comment) as number_of_comments
from site s;


-- comment view
create view comment_view as
with all_comment as 
(
    select
    c.*,
    (select board_id from post p where p.id = c.post_id),
    (select u.banned from user_ u where c.creator_id = u.id) as banned,
    (select bub.id::boolean from board_user_ban bub, post p where c.creator_id = bub.user_id and p.id = c.post_id and p.board_id = bub.board_id) as banned_from_board,
    (select name from user_ where c.creator_id = user_.id) as creator_name,
    coalesce(sum(cl.score), 0) as score,
    count (case when cl.score = 1 then 1 else null end) as upvotes,
    count (case when cl.score = -1 then 1 else null end) as downvotes
    from comment c 
    left join comment_like cl on c.id = cl.comment_id
    group by c.id
)

select
ac.*,
u.id as user_id,
coalesce(cl.score, 0) as my_vote,
(select cs.id::boolean from comment_saved cs where u.id = cs.user_id and cs.comment_id = ac.id) as saved
from user_ u 
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id

union all 

select
    ac.*,
    null as user_id,
    null as my_vote,
    null as saved
from all_comment ac
;

-- reply_view
create view reply_view as 
with closereply as (
    select
    c2.id,
    c2.creator_id as sender_id,
    c.creator_id as recipient_id
    from comment c 
    inner join comment c2 on c.id = c2.parent_id
    where c2.creator_id != c.creator_id
    -- doing a union where post is null
    union
    select
    c.id,
    c.creator_id as sender_id,
    p.creator_id as recipient_id
    from comment c, post p 
    where c.post_id = p.id and c.parent_id is null and c.creator_id != p.creator_id
)
select cv.*,
closereply.recipient_id
from comment_view cv, closereply
where closereply.id = cv.id 
;
