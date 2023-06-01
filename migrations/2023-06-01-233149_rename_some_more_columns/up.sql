alter table user_blocks rename to person_blocks;
alter table user_board_blocks rename to person_board_blocks;
alter table person_blocks rename user_id to person_id;
alter table person_board_blocks rename user_id to person_id;
alter table uploads rename user_id to person_id;
alter table registration_applications rename user_id to person_id;
alter table email_verification rename user_id to person_id;
alter table admin_purge_user rename user_id to person_id;
alter table mod_add_admin rename mod_user_id to mod_person_id;
alter table mod_add_admin rename other_user_id to other_person_id;

drop table private_messages cascade;