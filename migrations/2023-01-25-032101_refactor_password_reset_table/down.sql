drop table if exists password_resets;

create table password_reset_requests (
  id serial primary key,
  user_id int references users on update cascade on delete cascade not null,
  token_encrypted text not null,
  published timestamp not null default now()
);