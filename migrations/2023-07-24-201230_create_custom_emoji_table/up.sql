create table emoji (
    id serial primary key,
    local_site_id int references local_site on update cascade on delete cascade not null,
    shortcode varchar(128) not null UNIQUE,
    image_url text not null UNIQUE,
    alt_text text not null,
    category text not null,
    creation_date timestamp without time zone default now() not null,
    updated timestamp without time zone
);

create table emoji_keyword (
    id serial primary key,
    emoji_id int references emoji on update cascade on delete cascade not null,
    keyword varchar(128) not null,
    UNIQUE(emoji_id, keyword)
);

create index idx_emoji_category on emoji (id, category);