-- Remove board creation approval flag from person table
alter table person drop column board_creation_approved;

-- Remove board creation permission fields from local_site table
alter table local_site drop column trusted_user_min_posts;
alter table local_site drop column trusted_user_manual_approval;
alter table local_site drop column trusted_user_min_account_age_days;
alter table local_site drop column trusted_user_min_reputation;
alter table local_site drop column board_creation_mode;