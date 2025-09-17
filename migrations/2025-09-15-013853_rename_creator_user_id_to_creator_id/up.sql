-- Rename creator_user_id columns to creator_id for cleaner naming

-- Update comments table
ALTER TABLE comments RENAME COLUMN creator_user_id TO creator_id;

-- Update posts table
ALTER TABLE posts RENAME COLUMN creator_user_id TO creator_id;