alter table posts drop column is_stickied cascade;
alter table posts add column featured_board boolean not null default false;
alter table posts add column featured_local boolean not null default false;

alter table mod_sticky_post rename to mod_feature_post;
alter table mod_feature_post rename stickied to featured;