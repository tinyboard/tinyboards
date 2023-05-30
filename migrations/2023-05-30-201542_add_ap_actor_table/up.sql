create or replace function generate_unique_changeme()
returns text language sql
as $$
    select 'changeme_' || string_agg (substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil (random() * 62)::integer, 1), '')
    from generate_series(1, 20)
$$;

-- add apub columns for user
alter table users add column actor_id text not null default 'http://fake.com'; -- this should be checked and updated in code, build from site url if user is local
alter table users add column local boolean not null default true;
alter table users add column private_key text; -- keys generated in code
alter table users add column public_key text;
alter table users add column inbox_url text not null default generate_unique_changeme();
alter table users add column shared_inbox_url text;
alter table users add column bot_account boolean default false not null;
alter table users add column last_refreshed_date timestamp not null default now();
alter table users add constraint idx_person_inbox_url unique (inbox_url);

-- add apub columns for boards
alter table boards add column actor_id text not null default 'http://fake.com'; --needs to be checked and updated in code (built from site url if local)
alter table boards add column local not null default true;
alter table boards add column private_key text; -- keys managed in code
alter table boards add column public_key text;
alter table boards add column subscribers_url text not null default generate_unique_changeme();
alter table boards add column inbox_url text not null default generate_unique_changeme();
alter table boards add column shared_inbox_url text;
alter table boards add column last_refreshed_date timestamp not null default now();
alter table boards add constraint idx_board_subscribers_url unique (subscribers_url);
alter table boards add constraint idx_board_inbox_url unique (inbox_url);

-- rename users to person
alter table users rename to person;

alter table person add column local boolean default true


-- create a new local_user table
create table local_user(
    id serial primary key,
    person_id int references person on update cascade on delete cascade not null,
    passhash text not null,
    email text unique,
    is_admin boolean default false not null,
    show_nsfw boolean default false not null,
    show_bots boolean default true not null,
    theme text default '' not null,
    default_sort_type smallint default 0 not null,
    default_listing_type smallint default 1 not null,
    lang character varying(20) default 'browser'::character varying not null,
    email_notifications_enabled boolean default false not null,
    accepted_application boolean not null default false,
    is_application_accepted boolean not null default false,
    email_verified boolean not null default false,
    unique (person_id)
);

-- copy any local users into new table
insert into local_user
(
    person_id,
    passhash,
    email,
    is_admin,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    email_notifications_enabled,
    accepted_application,
    is_application_accepted,
    email_verified
)
select
    id,
    passhash,
    email,
    is_admin,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    email_notifications_enabled,
    accepted_application,
    is_application_accepted,
    email_verified
from person
where local = true;

alter table person
    drop column theme,
    drop column preferred_name,
    drop column passhash,
    drop column email,
    drop column is_admin,
    drop column show_nsfw,
    drop column theme,
    drop column default_sort_type,
    drop column default_listing_type,
    drop column email_notifications_enabled,
    drop column accepted_application,
    drop column is_application_accepted,
    drop column email_verified,
    drop column chat_id;

-- rename triggers
alter trigger site_aggregates_user_delete on person rename to site_aggregates_person_delete;
alter trigger site_aggregates_user_insert on person rename to site_aggregates_person_insert;

-- rename trigger functions
alter function site_aggregates_user_delete() rename to site_aggregates_person_delete;
alter function site_aggregates_user_insert() rename to site_aggregates_person_insert;

-- remake user_aggregates into person_aggregates
alter table user_aggregates rename to person_aggregates;
alter sequence user_aggregates_id_seq rename to person_aggregates_id_seq;
alter table person_aggregates rename column user_id to person_id;

-- drop old triggers and functions




