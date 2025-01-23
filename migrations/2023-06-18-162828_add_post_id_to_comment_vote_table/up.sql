alter table comment_votes add column post_id int references posts on update cascade on delete cascade not null default 1;

/*insert into comment_votes (post_id)
    select
        p.id
    from comments c
    left join posts p
        on p.id = c.post_id;*/

update comment_votes cv set post_id = (select post_id from comments where id = cv.comment_id) where true;

