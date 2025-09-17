-- Remove admin board ban log table
drop table admin_ban_board;

-- Remove admin board banning fields from boards table
alter table boards drop column banned_at;
alter table boards drop column banned_by;
alter table boards drop column public_ban_reason;
alter table boards drop column is_banned;