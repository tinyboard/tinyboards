-- This file should undo anything in `up.sql`
alter table local_site drop column color;
alter table local_site add column color varchar(6) default '3c6991';
