alter table board add column nsfw boolean default false not null;
alter table post add column nsfw boolean default false not null;
alter table user_ add column show_nsfw boolean default false not null;