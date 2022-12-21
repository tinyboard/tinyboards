alter table site_invite add column validated boolean default false not null;
insert into user_ (name, passhash) values ('admin', 'tinyboards');