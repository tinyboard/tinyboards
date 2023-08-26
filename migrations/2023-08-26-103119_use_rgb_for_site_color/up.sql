alter table local_site drop column color;
alter table local_site add column color varchar(12) default '60, 105, 145';
