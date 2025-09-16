-- Rename person_id columns to user_id across all affected tables

-- Update admin_purge_person table
ALTER TABLE admin_purge_person RENAME COLUMN person_id TO user_id;

-- Update mod tables (rename person_id references to user_id)
ALTER TABLE mod_add_admin RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_add_admin RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_add_board RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_add_board RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_add_board_mod RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_add_board_mod RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_ban RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_ban RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_ban_from_board RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_ban_from_board RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_feature_post RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_hide_board RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_lock_post RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_remove_board RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_remove_comment RENAME COLUMN mod_person_id TO mod_user_id;
ALTER TABLE mod_remove_post RENAME COLUMN mod_person_id TO mod_user_id;

-- Update person_blocks and person_board_blocks tables
ALTER TABLE person_blocks RENAME COLUMN person_id TO user_id;
ALTER TABLE person_board_blocks RENAME COLUMN person_id TO user_id;

-- Update person_subscriber table
ALTER TABLE person_subscriber RENAME COLUMN person_id TO user_id;

-- Update registration_applications table
ALTER TABLE registration_applications RENAME COLUMN person_id TO user_id;

-- Note: We skip person_aggregates and local_user tables as they will be dropped in favor of user_aggregates and users tables