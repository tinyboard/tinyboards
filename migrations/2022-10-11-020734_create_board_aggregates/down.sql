drop table board_aggregates;
drop trigger board_aggregates_board on board;
drop trigger board_aggregates_post_count on post;
drop trigger board_aggregates_comment_count on comment;
drop trigger board_aggregates_subscriber_count on board_subscriber;
drop function 
  board_aggregates_board,
  board_aggregates_post_count,
  board_aggregates_comment_count,
  board_aggregates_subscriber_count;