drop view board_user_ban_view;

alter table board_user_ban drop column expires;

create view board_user_ban_view as 
select *,
(select name from user_ u where bub.user_id = u.id) as user_name,
(select avatar from user_ u where bub.user_id = u.id),
(select name from board b where bub.board_id = b.id) as board_name
from board_user_ban bub;

