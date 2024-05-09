alter table local_user add column is_admin boolean not null default false;
update local_user set is_admin=true where admin_level > 0;
alter table local_user drop column admin_level;