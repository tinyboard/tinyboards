drop table admin_purge_user;

create table admin_purge_person (
    id serial primary key,
    admin_id int references person on update cascade on delete cascade not null,
    person_id int references person on update cascade on delete cascade not null,
    reason text,
    when_ timestamp not null default now()
);