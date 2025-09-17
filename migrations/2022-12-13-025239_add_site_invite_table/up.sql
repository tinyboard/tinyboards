create table site_invite (
    id serial primary key,
    email text not null,
    verification_code text not null,
    created timestamp not null default now()
);
