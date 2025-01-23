-- This is necessary because when bulk updating ranks (incrementing or decrementing all of them), the update happens in an arbitrary order and this trigger can cause it to fail.
-- Non-duplication is taken care of by triggers on this table.
alter table board_mods drop constraint unique_board_mods_board_id_rank;
