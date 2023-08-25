-- This file should undo anything in `up.sql`
alter table uploads drop column size;
alter table uploads add column size int default 0;
