create table private_message(
    id serial primary key,
    creator_id int references user_ on update cascade on delete cascade not null,
    recipient_id int references user_ on update cascade on delete cascade not null,
    body text not null,
    deleted boolean default false not null,
    read boolean default false not null,
    published timestamp not null default now(),
    updated timestamp
);