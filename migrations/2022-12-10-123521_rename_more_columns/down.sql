alter table posts rename column is_stickied to stickied;
alter table posts drop column updated;
alter table comments drop column updated;
alter table dms drop column updated;
alter table board_user_bans rename column creation_date to banned_date;
alter table board_subscriptions rename column creation_date to joined_date;
alter table board_mods rename column creation_date to added_date;

