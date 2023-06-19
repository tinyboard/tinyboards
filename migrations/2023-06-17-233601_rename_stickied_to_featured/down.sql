alter table posts drop column featured_board;
alter table posts drop column featured_local;
alter table posts add column is_stickied boolean not null default false;

alter table mod_feature_post rename to mod_sticky_post;
alter table mod_sticky_post rename featured to stickied;