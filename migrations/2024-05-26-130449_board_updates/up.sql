-- Add boards enabled column to local site
alter table local_site add column boards_enabled boolean not null default false;

-- Update boards to add sidebar html, ban reason and colors
alter table boards drop column is_banned;
alter table boards add column ban_reason varchar(512);
alter table boards add column primary_color varchar(25) not null default '60, 105, 145';
alter table boards add column secondary_color varchar(25) not null default '96, 128, 63';
alter table boards add column hover_color varchar(25) not null default '54, 94, 129';
alter table boards add column sidebar varchar(10000);
alter table boards add column sidebar_html text;
