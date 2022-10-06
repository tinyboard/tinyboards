alter table board add column deleted boolean default false not null;
alter table post add column deleted boolean default false not null;
alter table comment add column deleted boolean default false not null;