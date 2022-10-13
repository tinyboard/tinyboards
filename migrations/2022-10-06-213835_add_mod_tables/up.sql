create table mod_remove_post (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  post_id int references post on update cascade on delete cascade not null,
  reason text,
  removed boolean default true,
  when_ timestamp not null default now()
);

create table mod_lock_post (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  post_id int references post on update cascade on delete cascade not null,
  locked boolean default true,
  when_ timestamp not null default now()
);

create table mod_remove_comment (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  comment_id int references comment on update cascade on delete cascade not null,
  reason text,
  removed boolean default true,
  when_ timestamp not null default now()
);

create table mod_remove_board (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  board_id int references board on update cascade on delete cascade not null,
  reason text,
  removed boolean default true,
  expires timestamp,
  when_ timestamp not null default now()
);

-- TODO make sure you can't ban other mods
create table mod_ban_from_board (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  other_user_id int references user_ on update cascade on delete cascade not null,
  board_id int references board on update cascade on delete cascade not null,
  reason text,
  banned boolean default true,
  expires timestamp,
  when_ timestamp not null default now()
);

create table mod_ban (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  other_user_id int references user_ on update cascade on delete cascade not null,
  reason text,
  banned boolean default true,
  expires timestamp,
  when_ timestamp not null default now()
);

create table mod_add_board (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  other_user_id int references user_ on update cascade on delete cascade not null,
  board_id int references board on update cascade on delete cascade not null,
  removed boolean default false,
  when_ timestamp not null default now()
);

-- When removed is false that means kicked
create table mod_add (
  id serial primary key,
  mod_user_id int references user_ on update cascade on delete cascade not null,
  other_user_id int references user_ on update cascade on delete cascade not null,
  removed boolean default false,
  when_ timestamp not null default now()
);

