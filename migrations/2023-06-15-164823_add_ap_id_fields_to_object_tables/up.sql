alter table posts add column ap_id text;
alter table comments add column ap_id text;
alter table comments add column language_id int references language on update cascade on delete cascade not null default 0;