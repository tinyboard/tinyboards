drop table admin_purge_person;

create table admin_purge_user (
    id serial primary key,
    admin_id int references person on update cascade on delete cascade not null,
    person_id int references person on update cascade on delete cascade not null,
    reason text,
    when_ timestamp not null default now()
);