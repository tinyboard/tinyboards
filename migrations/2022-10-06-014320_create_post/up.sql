create table post (
    id serial primary key,
    name varchar(200) not null,
    type_ varchar(10) default 'text' not null, -- text | image | link
    url text,
    thumbnail_url text,
    permalink text,
    body text not null,
    creator_id int references user_ on update cascade on delete cascade not null,
    board_id int references board on update cascade on delete cascade not null,
    removed boolean default false not null,
    locked boolean default false not null,
    published timestamp not null default now(),
    updated timestamp
);

create table post_like (
    id serial primary key,
    post_id int references post on update cascade on delete cascade not null,
    user_id int references user_ on update cascade on delete cascade not null,
    score smallint not null, -- -1 or 1 for downvote, upvote respectfully
    unique(post_id, user_id)
);

create table post_saved (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(post_id, user_id)
);

create table post_read (
    id serial primary key,
    post_id int references post on update cascade on delete cascade not null,
    user_id int references user_ on update cascade on delete cascade not null,
    published timestamp not null default now(),
    unique(post_id, user_id)
);