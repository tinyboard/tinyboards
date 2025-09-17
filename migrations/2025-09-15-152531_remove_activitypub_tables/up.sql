-- Remove ActivityPub federation related tables since federation is no longer supported

-- Drop federation tables (they reference instance table)
DROP TABLE IF EXISTS federation_allowlist CASCADE;
DROP TABLE IF EXISTS federation_blocklist CASCADE;

-- Drop activity table (ActivityPub activities)
DROP TABLE IF EXISTS activity CASCADE;

-- Drop instance table (was used for federation)
DROP TABLE IF EXISTS instance CASCADE;

-- Remove federation_enabled field from local_site table
ALTER TABLE local_site DROP COLUMN IF EXISTS federation_enabled;