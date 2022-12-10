create table site_invite(
    id serial primary key,
    email text not null,
    token text not null,
    published timestamp not null default now()
);