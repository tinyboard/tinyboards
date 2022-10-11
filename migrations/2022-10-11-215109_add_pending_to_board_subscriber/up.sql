alter table board_subscriber add column pending boolean;

drop view board_subscriber_view;

create view board_subscriber_view as 
select *,
(select name from user_ u where bs.user_id = u.id) as user_name,
(select avatar from user_ u where bs.user_id = u.id),
(select name from board b where bs.board_id = b.id) as board_name,
(select pending from board_subscriber bs, user_ u, board b where bs.user_id = u.id and bs.board_id = b.id) as subscription_pending
from board_subscriber bs;
