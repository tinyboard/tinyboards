create table admin_purge_user (
    id serial primary key,
    admin_id int references user_ on update cascade on delete cascade not null,
    user_id int references user_ on update cascade on delete cascade not null,
    reason text,
    when_ timestamp not null default now()
);

create table admin_purge_board (
    id serial primary key,
    admin_id int references user_ on update cascade on delete cascade not null,
    board_id int references board on update cascade on delete cascade not null,
    reason text,
    when_ timestamp not null default now()
);

create table admin_purge_post (
    id serial primary key,
    admin_id int references user_ on update cascade on delete cascade not null,
    post_id int references post on update cascade on delete cascade not null,
    reason text,
    when_ timestamp not null default now()
);

create table admin_purge_comment (
    id serial primary key,
    admin_id int references user_ on update cascade on delete cascade not null,
    comment_id int references comment on update cascade on delete cascade not null,
    reason text,
    when_ timestamp not null default now()
);