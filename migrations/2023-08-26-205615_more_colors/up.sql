-- also description lol
alter table local_site drop column color;
alter table local_site add column primary_color varchar(25) default '60, 105, 145';
alter table local_site add column secondary_color varchar(25) default '96, 128, 63';
alter table local_site add column hover_color varchar(25) default '54, 94, 129';
alter table local_site add column description varchar(255);
