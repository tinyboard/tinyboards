drop table person_subscriber;
alter table board_subscriber alter column pending drop not null;
alter table board_subscriber rename to board_subscriptions;