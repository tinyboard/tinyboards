alter table user_ add column banner text;
alter table user_ add column bio text;
alter table user_ drop column fedi_name cascade; -- we are not federating at this point