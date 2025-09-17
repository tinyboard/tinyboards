create table mod_hide_board(
    id serial primary key,
    board_id int references boards on update cascade on delete cascade not null,
    mod_person_id int references person on update cascade on delete cascade not null,
    when_ timestamp not null default now(),
    reason text,
    hidden boolean not null default false
);