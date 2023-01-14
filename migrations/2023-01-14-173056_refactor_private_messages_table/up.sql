drop table private_messages cascade;

create table private_messages(
    id serial primary key,
    chat_id text not null,
    creator_id int references users on update cascade on delete cascade not null,
    recipient_id int references users on update cascade on delete cascade not null,
    subject text,
    body text not null,
    is_parent boolean default false not null,
    is_deleted boolean default false not null,
    read boolean default false not null,
    creation_date timestamp not null default now(),
    updated timestamp
);