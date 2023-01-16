alter table private_messages add column chat_id text not null;
alter table private_messages add column subject text;

alter table users drop column chat_id cascade;