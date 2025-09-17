alter table local_user add column admin_level integer not null default 0;
update local_user set admin_level=64 where is_admin=true;
alter table local_user drop column is_admin;