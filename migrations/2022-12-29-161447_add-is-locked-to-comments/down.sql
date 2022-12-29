-- This file should undo anything in `up.sql`
alter table comments drop column is_locked;
