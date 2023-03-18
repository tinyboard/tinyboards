CREATE TABLE uploads (
    id serial primary key,
    filename text not null,
    filepath text not null,
    uploaded_at timestamp not null default now()
);