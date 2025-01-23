create table stray_images (
    id serial primary key,
    img_url text not null,
    creation_date timestamp not null default now()
);