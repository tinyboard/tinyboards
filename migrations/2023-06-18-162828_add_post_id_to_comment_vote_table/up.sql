alter table comment_votes add column post_id int references posts on update cascade on delete cascade not null;

insert into comment_votes (post_id)
    select
        p.id
    from comments c
    left join posts p
        on p.id = c.post_id;

