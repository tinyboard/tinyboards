-- This file should undo anything in `up.sql`
alter table board_mods drop constraint unique_board_mods_board_id_rank;

drop trigger board_mods_before_insert on board_mods cascade;
drop trigger board_mods_after_delete on board_mods cascade;

--drop function board_mods_insert_set_rank;
--drop function board_mods_delete_update_ranks;
