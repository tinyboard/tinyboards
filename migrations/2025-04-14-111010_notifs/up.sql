alter table private_message add column is_sender_hidden boolean not null default false;
alter table private_message add column title text not null default '<Untitled>';

drop table person_mentions;
drop table comment_reply;


create table notifications (
	id serial primary key,
	kind text not null,
	recipient_id int references local_user on update cascade on delete cascade not null,
	comment_id int references comments on update cascade,
	post_id int references posts on update cascade,
	message_id int references private_message on update cascade,
	created timestamp not null default now(),
	is_read boolean not null default false
);
