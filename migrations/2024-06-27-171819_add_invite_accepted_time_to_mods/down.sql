-- This file should undo anything in `up.sql`
alter table board_mods drop constraint unique_board_mods_board_id_person_id;

drop trigger board_mods_before_insert_2 on board_mods cascade;

alter table board_mods drop column invite_accepted_date;
