-- Remove last_seen column and index
DROP INDEX IF EXISTS idx_users_last_seen;
ALTER TABLE users DROP COLUMN IF EXISTS last_seen;
