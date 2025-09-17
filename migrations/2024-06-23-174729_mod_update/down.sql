alter table board_mods drop column permissions;
alter table board_mods drop column rank;
alter table board_mods drop column invite_accepted;
-- This file should undo anything in `up.sql`