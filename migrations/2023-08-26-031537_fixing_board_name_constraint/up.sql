ALTER TABLE boards DROP CONSTRAINT board_name_key;
ALTER TABLE boards ADD CONSTRAINT board_name_and_instance_id_unique UNIQUE(name, instance_id);
ALTER TABLE local_user ADD CONSTRAINT local_user_name_unique UNIQUE(name);