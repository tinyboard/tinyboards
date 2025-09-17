-- Add board creation permission fields to local_site table
alter table local_site add column board_creation_mode varchar(20) not null default 'admin_only';
alter table local_site add column trusted_user_min_reputation integer not null default 100;
alter table local_site add column trusted_user_min_account_age_days integer not null default 30;
alter table local_site add column trusted_user_manual_approval boolean not null default false;
alter table local_site add column trusted_user_min_posts integer not null default 5;

-- Add board creation approval flag to person table
alter table person add column board_creation_approved boolean not null default false;

-- Set existing board_creation_admin_only sites to appropriate mode
update local_site set board_creation_mode = 'admin_only' where board_creation_admin_only = true;
update local_site set board_creation_mode = 'open' where board_creation_admin_only = false;