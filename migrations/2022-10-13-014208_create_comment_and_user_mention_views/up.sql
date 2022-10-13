-- comment view
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

create view user_mention_view as
select 
    c.id,
    um.id as user_mention_id,
    c.creator_id,
    c.post_id,
    c.parent_id,
    c.body,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.board_id,
    c.banned,
    c.user_banned_from_board,
    c.creator_name,
    c.creator_avatar,
    c.score,
    c.upvotes,
    c.downvotes,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id
from user_mention um, comment_view c
where um.comment_id = c.id;