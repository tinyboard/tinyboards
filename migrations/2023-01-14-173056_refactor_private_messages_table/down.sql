drop table private_messages cascade;

create table private_messages(
    id serial primary key,
    creator_id int references users on update cascade on delete cascade not null,
    recipient_id int references users on update cascade on delete cascade not null,
    subject text,
    body text not null,
    is_deleted boolean default false not null,
    read boolean default false not null,
    creation_date timestamp not null default now(),
    updated timestamp,
    parent_id int references private_messages on update cascade on delete cascade
);