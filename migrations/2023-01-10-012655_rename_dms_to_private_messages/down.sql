drop table private_messages;

create table dms (
    id serial primary key,
    creator_id int references users on update cascade on delete cascade not null,
    recipient_id int references users on update cascade on delete cascade not null,
    body text not null,
    is_deleted boolean default false not null,
    read boolean default false not null,
    creation_date timestamp not null default now(),
    updated timestamp
);