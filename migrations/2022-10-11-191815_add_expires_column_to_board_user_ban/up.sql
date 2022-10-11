alter table board_user_ban add column expires timestamp;

-- remake the view
drop view board_user_ban_view;

create view board_user_ban_view as 
select *,
(select name from user_ u where bub.user_id = u.id) as user_name,
(select avatar from user_ u where bub.user_id = u.id),
(select name from board b where bub.board_id = b.id) as board_name,
(select expires from board_user_ban bub, user_ u, board b where u.id = bub.user_id and b.id = bub.board_id) as ban_expires
from board_user_ban bub;