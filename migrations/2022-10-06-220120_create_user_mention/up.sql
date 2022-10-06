create table user_mention (
    id serial primary key,
    recipient_id int references user_ on update cascade on delete cascade not null,
    comment_id int references comment on update cascade on delete cascade not null,
    read boolean default false not null,
    published timestamp not null default now(),
    unique(recipient_id, comment_id)
);