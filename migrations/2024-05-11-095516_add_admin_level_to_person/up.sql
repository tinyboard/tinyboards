alter table person add column admin_level integer not null default 0;
update person set admin_level=64 where is_admin=true;
--alter table person drop column is_admin;