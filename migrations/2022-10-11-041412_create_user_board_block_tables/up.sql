create table user_block (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  target_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(user_id, target_id)
);

create table board_block (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  board_id int references board on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(user_id, board_id)
);