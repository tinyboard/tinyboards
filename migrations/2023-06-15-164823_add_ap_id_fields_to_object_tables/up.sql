alter table posts add column ap_id text;
alter table posts add column local boolean not null default true;
alter table comments add column local boolean not null default true;
alter table comments add column ap_id text;
alter table comments add column language_id int references language on update cascade on delete cascade not null default 0;