-- Your SQL goes here
alter table uploads drop column size;
alter table uploads add column size bigint not null default 0;
