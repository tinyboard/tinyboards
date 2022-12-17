alter table site_invite add column validated boolean default false not null;
alter table site_invite rename column created to published;