alter table user_ add column deleted boolean default false not null;
alter table user_ add column expires timestamp;