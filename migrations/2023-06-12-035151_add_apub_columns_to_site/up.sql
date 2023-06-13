alter table site rename column description to sidebar;
alter table site drop column creator_id cascade;
alter table site add column icon text;
alter table site add column banner text;
alter table site add column description text;
alter table site add column last_refreshed_date timestamp not null default now();
alter table site add column inbox_url text not null;
alter table site add column private_key text;
alter table site add column public_key text not null;