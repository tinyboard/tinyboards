-- This file should undo anything in `up.sql`
alter table local_site add column color varchar(12) default '60, 105, 145';
alter table local_site drop column primary_color;
alter table local_site drop column secondary_color;
alter table local_site drop column hover_color;
alter table local_site drop column description;