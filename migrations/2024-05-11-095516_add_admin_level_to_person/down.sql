--alter table person add column is_admin boolean not null default false;
--update person set is_admin=true where admin_level > 0;
alter table person drop column admin_level;