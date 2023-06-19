create table person_subscriber (
    id serial primary key,
    person_id int references person on update cascade on delete cascade not null,
    subscriber_id int references person on update cascade on delete cascade not null,
    creation_date timestamp not null default now(),
    pending boolean not null,
    unique(subscriber_id, person_id)
);

alter table board_subscriptions rename to board_subscriber;
update board_subscriber set pending = false where pending is null;
alter table board_subscriber alter column pending set not null;