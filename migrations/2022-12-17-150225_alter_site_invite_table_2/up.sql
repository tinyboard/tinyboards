alter table site_invite drop column validated;
delete from user_ where name = 'admin';