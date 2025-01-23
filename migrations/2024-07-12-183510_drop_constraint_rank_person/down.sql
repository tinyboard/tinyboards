-- Undo drop constraint.
alter table board_mods
	add constraint unique_board_mods_board_id_rank
	unique(board_id, rank);
