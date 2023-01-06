create table notifications (
    id serial primary key,
    user_id integer not null references users(id),
    comment_id integer not null references comments(id),
    creation_date timestamp not null default now(),
    is_read boolean not null default false
);
