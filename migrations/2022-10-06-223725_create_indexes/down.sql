drop index if exists idx_post_creator;
drop index if exists idx_post_board;

drop index if exists idx_post_like_post;
drop index if exists idx_post_like_user;

drop index if exists idx_comment_creator;
drop index if exists idx_comment_parent;
drop index if exists idx_comment_post;

drop index if exists idx_comment_like_comment;
drop index if exists idx_comment_like_user;
drop index if exists idx_comment_like_post;

drop index if exists idx_board_creator;
drop index if exists idx_board_tag;