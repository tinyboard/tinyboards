-- This file should undo anything in `up.sql`
alter table board_mods alter column invite_accepted_date set default null;