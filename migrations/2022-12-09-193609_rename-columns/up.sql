-- rename a bunch of columns/tables
alter table user_ rename to users;
alter table board_subscriber rename to board_subscriptions;
alter table board_user_ban rename to board_user_bans;
alter table board_moderator rename to board_mods;
alter table board rename to boards;
alter table post rename to posts;
alter table post_vote rename to post_votes;
alter table comment_vote rename to comment_votes;
alter table password_reset_request rename to password_reset_requests;
alter table private_message rename to dms;
alter table user_block rename to user_blocks;
alter table board_block rename to user_board_blocks;
alter table registration_application rename to registration_applications;
alter table post_saved rename to user_post_save;
alter table post_read rename to user_post_read;
alter table comment rename to comments;
alter table comment_saved rename to user_comment_save;
alter table user_mention rename to user_mentions;

alter table users rename column admin to is_admin;
alter table users rename column banned to is_banned;
alter table users rename column published to creation_date;
alter table users rename column deleted to is_deleted;
alter table users rename column expires to unban_date;
alter table users rename column application_accepted to is_application_accepted;

alter table boards rename column removed to is_banned;
alter table boards rename column deleted to is_deleted;
alter table boards rename column published to creation_date;
alter table boards rename column hidden to is_hidden;

alter table board_mods rename column published to added_date;
alter table board_subscriptions rename column published to joined_date;
alter table board_user_bans rename column published to banned_date;

alter table posts rename column removed to is_removed;
alter table posts rename column deleted to is_deleted;
alter table posts rename column locked to is_locked;
alter table posts rename column updated to edited_date;
alter table posts rename column nsfw to is_nsfw;

alter table user_post_save rename column published to creation_date;
alter table user_post_read rename column published to creation_date;

alter table comments rename column removed to is_removed;
alter table comments rename column deleted to is_deleted;
alter table comments rename column published to creation_date;
alter table comments rename column updated to edited_date;

alter table comment_votes rename column published to creation_date;
alter table user_comment_save rename column published to creation_date;

alter table dms rename column deleted to is_deleted;
alter table dms rename column published to creation_date;
alter table dms rename column updated to edited_date;

alter table user_blocks rename column published to creation_date;
alter table user_board_blocks rename column published to creation_date;

