create view mod_remove_post_view as 
select mrp.*,
(select name from user_ u where mrp.mod_user_id = u.id) as mod_user_name,
(select name from post p where mrp.post_id = p.id) as post_name,
(select b.id from post p, board b where mrp.post_id = p.id and p.board_id = b.id) as board_id,
(select b.name from post p, board b where mrp.post_id = p.id and p.board_id = b.id) as board_name
from mod_remove_post mrp;

create view mod_lock_post_view as 
select mlp.*,
(select name from user_ u where mlp.mod_user_id = u.id) as mod_user_name,
(select name from post p where mlp.post_id = p.id) as post_name,
(select b.id from post p, board b where mlp.post_id = p.id and p.board_id = b.id) as board_id,
(select b.name from post p, board b where mlp.post_id = p.id and p.board_id = b.id) as board_name
from mod_lock_post mlp;

create view mod_remove_comment_view as 
select mrc.*,
(select name from user_ u where mrc.mod_user_id = u.id) as mod_user_name,
(select c.id from comment c where mrc.comment_id = c.id) as comment_user_id,
(select name from user_ u, comment c where mrc.comment_id = c.id and u.id = c.creator_id) as comment_user_name,
(select body from comment c where mrc.comment_id = c.id) as comment_body,
(select p.id from post p, comment c where mrc.comment_id = c.id and c.post_id = p.id) as post_id,
(select p.name from post p, comment c where mrc.comment_id = c.id and c.post_id = p.id) as post_name,
(select b.id from comment c, post p, board b where mrc.comment_id = c.id and c.post_id = p.id and p.board_id = b.id) as board_id, 
(select b.name from comment c, post p, board b where mrc.comment_id = c.id and c.post_id = p.id and p.board_id = b.id) as board_name
from mod_remove_comment mrc;

create view mod_remove_board_view as 
select mrb.*,
(select name from user_ u where mrb.mod_user_id = u.id) as mod_user_name,
(select b.name from board b where mrb.board_id = b.id) as board_name
from mod_remove_board mrb;

create view mod_ban_from_board_view as 
select mb.*,
(select name from user_ u where mb.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where mb.other_user_id = u.id) as other_user_name,
(select name from board b where mb.board_id = b.id) as board_name
from mod_ban_from_board mb;

create view mod_ban_view as 
select mb.*,
(select name from user_ u where mb.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where mb.other_user_id = u.id) as other_user_name
from mod_ban mb;

create view mod_add_board_view as 
select ma.*,
(select name from user_ u where ma.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where ma.other_user_id = u.id) as other_user_name,
(select name from board b where ma.board_id = b.id) as board_name
from mod_add_board ma;

create view mod_add_view as 
select ma.*,
(select name from user_ u where ma.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where ma.other_user_id = u.id) as other_user_name
from mod_add ma;