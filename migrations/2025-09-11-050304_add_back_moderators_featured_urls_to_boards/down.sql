-- Remove moderators_url and featured_url from boards table
ALTER TABLE boards DROP COLUMN IF EXISTS moderators_url;
ALTER TABLE boards DROP COLUMN IF EXISTS featured_url;
