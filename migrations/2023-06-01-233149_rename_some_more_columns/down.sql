alter table person_blocks rename to user_blocks;
alter table person_board_blocks rename to user_board_blocks;
alter table user_blocks rename person_id to user_id;
alter table user_board_blocks rename person_id to user_id;
alter table uploads rename person_id to user_id;
alter table registration_applications rename person_id to user_id;
alter table email_verification rename person_id to user_id;
alter table admin_purge_user rename person_id to user_id;
alter table mod_add_admin rename mod_person_id to mod_user_id;
alter table mod_add_admin rename other_person_id to other_user_id;

create table private_messages(
    id serial primary key,
    chat_id text not null,
    creator_id int references person on update cascade on delete cascade not null,
    recipient_id int references person on update cascade on delete cascade not null,
    subject text,
    body text not null,
    is_parent boolean default false not null,
    is_deleted boolean default false not null,
    read boolean default false not null,
    creation_date timestamp not null default now(),
    updated timestamp
);