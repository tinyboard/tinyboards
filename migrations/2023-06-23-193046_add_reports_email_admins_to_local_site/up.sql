alter table local_site add column reports_email_admins boolean not null default true;

update local_site set reports_email_admins = true where id = 1;