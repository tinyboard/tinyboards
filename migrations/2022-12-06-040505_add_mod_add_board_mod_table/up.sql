create table mod_add_board_mod (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  other_user_id int references user_ on update cascade on delete cascade not null,
  board_id int references board on update cascade on delete cascade not null,
  removed boolean default false,
  when_ timestamp not null default now()
);

alter table mod_add rename to mod_add_admin;