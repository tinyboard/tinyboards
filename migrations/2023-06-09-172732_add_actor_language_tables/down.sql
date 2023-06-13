alter table posts drop column language_id;
drop table local_user_language;
drop table language;

alter table local_user rename column interface_language to lang;

alter table person add column login_nonce integer default 0;
alter table person drop column is_admin;