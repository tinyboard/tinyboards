ALTER TABLE boards DROP CONSTRAINT board_name_and_instance_id_unique;
ALTER TABLE boards ADD CONSTRAINT board_name_key UNIQUE(name);
ALTER TABLE local_user DROP CONSTRAINT local_user_name_unique;