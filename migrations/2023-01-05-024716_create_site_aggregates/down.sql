drop table site_aggregates;
drop trigger site_aggregates_site on site;
drop trigger site_aggregates_user_insert on users;
drop trigger site_aggregates_user_delete on users;
drop trigger site_aggregates_post_insert on posts;
drop trigger site_aggregates_post_delete on posts;
drop trigger site_aggregates_comment_insert on comments;
drop trigger site_aggregates_comment_delete on comments;
drop trigger site_aggregates_board_insert on boards;
drop trigger site_aggregates_board_delete on boards;
drop function 
  site_aggregates_site,
  site_aggregates_user_insert,
  site_aggregates_user_delete,
  site_aggregates_post_insert,
  site_aggregates_post_delete,
  site_aggregates_comment_insert,
  site_aggregates_comment_delete,
  site_aggregates_board_insert,
  site_aggregates_board_delete;