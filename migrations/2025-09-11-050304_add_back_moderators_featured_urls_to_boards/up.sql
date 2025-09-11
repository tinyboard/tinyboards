-- Add back moderators_url and featured_url to boards table
ALTER TABLE boards ADD COLUMN moderators_url TEXT;
ALTER TABLE boards ADD COLUMN featured_url TEXT;
