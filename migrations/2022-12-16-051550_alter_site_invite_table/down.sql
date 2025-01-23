alter table site_invite drop column validated;
alter table site_invite add column email text not null;