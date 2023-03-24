create table uploads (
    id serial primary key,
    user_id int references users on update cascade on delete cascade not null,
    original_name text not null,
    file_name text not null,
    file_path text not null,
    creation_date timestamp not null default now()
);