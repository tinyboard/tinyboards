-- Revert person_id column renames back to original names

-- Revert admin_purge_person table
ALTER TABLE admin_purge_person RENAME COLUMN user_id TO person_id;

-- Revert mod tables
ALTER TABLE mod_add_admin RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_add_admin RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_add_board RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_add_board RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_add_board_mod RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_add_board_mod RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_ban RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_ban RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_ban_from_board RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_ban_from_board RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_feature_post RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_hide_board RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_lock_post RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_remove_board RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_remove_comment RENAME COLUMN mod_user_id TO mod_person_id;
ALTER TABLE mod_remove_post RENAME COLUMN mod_user_id TO mod_person_id;

-- Revert person_blocks and person_board_blocks tables
ALTER TABLE person_blocks RENAME COLUMN user_id TO person_id;
ALTER TABLE person_board_blocks RENAME COLUMN user_id TO person_id;

-- Revert person_subscriber table
ALTER TABLE person_subscriber RENAME COLUMN user_id TO person_id;

-- Revert registration_applications table
ALTER TABLE registration_applications RENAME COLUMN user_id TO person_id;