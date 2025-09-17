-- The ActivityPub Activity Table
-- all user actions must ultimately generate a row here
create table activity (
    id serial primary key,
    ap_id text not null,
    data jsonb not null,
    local boolean not null default true,
    sensitive boolean not null default true,
    creation_date timestamp not null default now(),
    updated timestamp
);