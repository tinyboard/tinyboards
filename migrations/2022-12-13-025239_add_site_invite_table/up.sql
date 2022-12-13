create table site_invite (
    id serial primary key,
    user_id int references user_ on update cascade on delete cascade not null,
    email text not null,
    verification_code text not null,
    created timestamp not null default now()
);
