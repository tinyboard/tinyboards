alter table site drop column require_application;
alter table site drop column application_question;
alter table site drop column private_instance;

alter table user_ drop column accepted_application;

drop table registration_application;