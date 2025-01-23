create table comment_reply (
    id serial primary key,
    recipient_id int references users on update cascade on delete cascade not null,
    comment_id int references comments on update cascade on delete cascade not null,
    read boolean default false not null,
    creation_date timestamp not null default now(),
    unique(recipient_id, comment_id)
);

-- ones where parent_id is null, use post creator recipient
insert into comment_reply (recipient_id, comment_id, read)
select p.creator_id, c.id, c.read from comments c
inner join posts p on c.post_id = p.id
where c.parent_id is null;

-- Ones where there is a parent_id, self join to comment for parent comment creator
insert into comment_reply (recipient_id, comment_id, read)
select c2.creator_id, c.id, c.read from comments c 
inner join comments c2 on c.parent_id = c2.id;