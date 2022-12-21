alter table site_invite drop column email;
alter table site_invite add column validated boolean default false not null;