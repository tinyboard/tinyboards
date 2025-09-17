-- A user can be on a board's mod list only once (wow)
alter table board_mods
	add constraint unique_board_mods_board_id_person_id
	unique(board_id, person_id);

alter table board_mods add column invite_accepted_date timestamp;

update board_mods set invite_accepted_date = now()
	where invite_accepted_date is null
	and invite_accepted = true;

create or replace function set_invite_accepted_date()
	returns trigger
	language plpgsql
as
$$
begin
	if NEW.invite_accepted = true and OLD.invite_accepted = false then
		NEW.invite_accepted_date := now();
	end if;

	return NEW;
end;
$$;

create trigger board_mods_before_update
	before update
	on board_mods
	for each row
	execute function set_invite_accepted_date();