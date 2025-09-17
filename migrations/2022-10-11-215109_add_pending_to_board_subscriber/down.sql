drop view board_subscriber_view;

alter table board_subscriber drop column pending;

create view board_subscriber_view as 
select *,
(select name from user_ u where bs.user_id = u.id) as user_name,
(select avatar from user_ u where bs.user_id = u.id),
(select name from board b where bs.board_id = b.id) as board_name
from board_subscriber bs;