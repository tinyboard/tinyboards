-- This file should undo anything in `up.sql`
alter table local_site drop column name;
alter table local_site drop column color;
