alter table boards add column moderators_url text;
alter table boards add column featured_url text;
alter table boards add column icon text;
alter table boards add column banner text;
alter table boards add column posting_restricted_to_mods boolean not null default false;