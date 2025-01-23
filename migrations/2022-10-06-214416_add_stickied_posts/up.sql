-- Add column
alter table post add column stickied boolean default false not null;

-- Add mod table
create table mod_sticky_post (
    id serial primary key,
    mod_user_id int references user_ on update cascade on delete cascade not null,
    post_id int references post on update cascade on delete cascade not null,
    stickied boolean default true,
    when_ timestamp not null default now()
);
