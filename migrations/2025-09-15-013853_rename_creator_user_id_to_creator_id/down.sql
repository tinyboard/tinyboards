-- Revert creator_id columns back to creator_user_id

-- Revert comments table
ALTER TABLE comments RENAME COLUMN creator_id TO creator_user_id;

-- Revert posts table
ALTER TABLE posts RENAME COLUMN creator_id TO creator_user_id;