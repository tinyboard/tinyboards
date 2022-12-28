create table reports (
  id serial primary key,
  user_id int references users,
  comment_id int references comments,
  post_id int references posts,
  reason text,
  creation_date timestamp not null default now()
);
