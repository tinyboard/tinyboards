create table comment_report(
    id serial primary key,
    creator_id int references person on update cascade on delete cascade not null,
    comment_id int references comments on update cascade on delete cascade not null,
    original_comment_text text not null,
    reason text not null,
    resolved boolean not null default false,
    resolver_id int references person on update cascade on delete cascade not null,
    creation_date timestamp not null default now(),
    updated timestamp null,
    unique(comment_id, creator_id)
);

create table post_report(
    id serial primary key,
    creator_id int references person on update cascade on delete cascade not null,
    post_id int references posts on update cascade on delete cascade not null,
    original_post_title text not null,
    original_post_url text,
    original_post_body text,
    reason text not null,
    resolved bool not null default false,
    resolver_id int references person on update cascade on delete cascade not null,
    creation_date timestamp not null default now(),
    updated timestamp null,
    unique (post_id, creator_id)
);