alter table posts rename column stickied to is_stickied;
alter table posts add column updated timestamp;

alter table comments add column updated timestamp;

alter table dms add column updated timestamp;

alter table board_user_bans rename column banned_date to creation_date;

alter table board_subscriptions rename column joined_date to creation_date;

alter table board_mods rename column added_date to creation_date;

