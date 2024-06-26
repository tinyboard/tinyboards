-- there can be no mods with the same rank on the same board
alter table board_mods
	add constraint unique_board_mods_board_id_rank
	unique(board_id, rank);

-- when a new mod is inserted, they get assigned a rank: below the formerly lowest-ranking mod
create or replace function board_mods_insert_set_rank()
	returns trigger
	language plpgsql
as
$$
declare
	lowest_rank int;
begin
	select rank into lowest_rank
		from board_mods
		where board_id = NEW.board_id
		order by rank desc
		limit 1;

	if lowest_rank is null then
		lowest_rank := 0;
	end if;

	NEW.rank := lowest_rank + 1;

	return NEW;
end;
$$;

create trigger board_mods_before_insert
	before insert
	on board_mods
	for each row
	execute function board_mods_insert_set_rank();

-- when a mod is deleted, the mods below them climb one rank higher
create or replace function board_mods_delete_update_ranks()
	returns trigger
	language plpgsql
as
$$
begin
	update board_mods
		set rank = rank - 1
		where board_id = OLD.board_id
		and rank >= OLD.rank;

	return OLD;
end;
$$;

create trigger board_mods_after_delete
	after delete
	on board_mods
	for each row
	execute function board_mods_delete_update_ranks();
