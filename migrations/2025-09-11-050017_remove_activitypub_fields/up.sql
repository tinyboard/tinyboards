-- Remove all ActivityPub/federation fields since we're not federating

-- Remove from person table
ALTER TABLE person DROP COLUMN IF EXISTS actor_id;
ALTER TABLE person DROP COLUMN IF EXISTS local;
ALTER TABLE person DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE person DROP COLUMN IF EXISTS shared_inbox_url;

-- Remove from boards table  
ALTER TABLE boards DROP COLUMN IF EXISTS actor_id;
ALTER TABLE boards DROP COLUMN IF EXISTS local;
ALTER TABLE boards DROP COLUMN IF EXISTS subscribers_url;
ALTER TABLE boards DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE boards DROP COLUMN IF EXISTS shared_inbox_url;
ALTER TABLE boards DROP COLUMN IF EXISTS moderators_url;
ALTER TABLE boards DROP COLUMN IF EXISTS featured_url;

-- Remove from site table
ALTER TABLE site DROP COLUMN IF EXISTS actor_id;
ALTER TABLE site DROP COLUMN IF EXISTS inbox_url;

-- Remove from comments table
ALTER TABLE comments DROP COLUMN IF EXISTS local;
ALTER TABLE comments DROP COLUMN IF EXISTS ap_id;

-- Remove from posts table
ALTER TABLE posts DROP COLUMN IF EXISTS ap_id;
ALTER TABLE posts DROP COLUMN IF EXISTS local;