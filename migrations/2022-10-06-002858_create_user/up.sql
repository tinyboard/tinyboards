create table user_ (
  id serial primary key,
  name varchar(30) not null,
  fedi_name varchar(40),
  preferred_name varchar(30),
  passhash text not null,
  email text unique,
  admin boolean default false not null,
  banned boolean default false not null,
  published timestamp not null default now(),
  updated timestamp,
  theme varchar(20) default 'dark' not null,
  default_sort_type smallint default 0 not null,
  default_listing_type smallint default 1 not null,
  avatar text,
  email_notifications_enabled boolean default false not null,
  unique(name)
);

create table user_ban (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique (user_id)
);

insert into user_ (name, passhash) values ('admin', 'porpl');

create unique index idx_user_name_lower on user_ (lower(name));
create unique index idx_user_email_lower on user_ (lower(email));