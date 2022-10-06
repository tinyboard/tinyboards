create table user_ (
  id serial primary key,
  name varchar(30) not null,
  fedi_name varchar(40) not null,
  preferred_name varchar(30),
  passhash text not null,
  email text unique,
  admin boolean default false not null,
  banned boolean default false not null,
  published timestamp not null default now(),
  updated timestamp,
  theme varchar(20) default 'dark' not null,
  unique(name, fedi_name)
);

create table user_ban (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique (user_id)
);

insert into user_ (name, fedi_name, passhash) values ('admin', 'porpl', 'porpl');