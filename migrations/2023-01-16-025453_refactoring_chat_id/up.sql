alter table private_messages drop column chat_id;
alter table private_messages drop column subject;

alter table users add column chat_id text not null default 'n/a';