drop table if exists password_reset_requests;

create table password_resets (
    id serial primary key,
    user_id int references users on update cascade on delete cascade not null,
    reset_token text not null,
    creation_date timestamp not null default now()
);