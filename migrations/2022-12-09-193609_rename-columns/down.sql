alter table users rename to user_;
alter table board_subscriptions rename to board_subscriber;
alter table board_user_bans rename to board_user_ban;
alter table board_mods rename to board_moderator;
alter table boards rename to board;
alter table posts rename to post;
alter table post_votes rename to post_vote;
alter table comment_votes rename to comment_vote;
alter table password_reset_requests rename to password_reset_request;
alter table dms rename to private_message;
alter table user_blocks rename to user_block;
alter table user_board_blocks rename to board_block;
alter table registration_applications rename to registration_application;
alter table user_post_save rename to post_saved;
alter table user_post_read rename to post_read;
alter table comments rename to comment;
alter table user_comment_save rename to comment_saved;
alter table user_mentions rename to user_mention;

alter table user_ rename column is_admin to admin;
alter table user_ rename column is_banned to banned;
alter table user_ rename column creation_date to published;
alter table user_ rename column is_deleted to deleted;
alter table user_ rename column unban_date to expires;
alter table user_ rename column is_application_accepted to application_accepted;

alter table board rename column is_banned to removed;
alter table board rename column is_deleted to deleted;
alter table board rename column creation_date to published;
alter table board rename column is_hidden to hidden;

alter table board_moderator rename column added_date to published;
alter table board_subscriber rename column joined_date to published;
alter table board_user_ban rename column banned_date to published;

alter table post rename column is_removed to removed;
alter table post rename column is_deleted to deleted;
alter table post rename column is_locked to locked;
alter table post rename column edited_date to updated;
alter table post rename column is_nsfw to nsfw;

alter table post_saved rename column creation_date to published;
alter table post_read rename column creation_date to published;

alter table comment rename column is_removed to removed;
alter table comment rename column is_deleted to deleted;
alter table comment rename column creation_date to published;
alter table comment rename column edited_date to updated;

alter table comment_vote rename column creation_date to published;
alter table comment_saved rename column creation_date to published;

alter table private_message rename column is_deleted to deleted;
alter table private_message rename column creation_date to published;
alter table private_message rename column edited_date to updated;

alter table user_block rename column creation_date to published;
alter table board_block rename column creation_date to published;

