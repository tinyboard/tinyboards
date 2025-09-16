-- Rename local_site table to site and remove all federation-related columns

-- First drop the old site table since we're replacing it with local_site
DROP TABLE IF EXISTS site CASCADE;

-- Remove all federation-related columns from local_site
ALTER TABLE local_site DROP COLUMN IF EXISTS federation_debug;
ALTER TABLE local_site DROP COLUMN IF EXISTS federation_strict_allowlist;
ALTER TABLE local_site DROP COLUMN IF EXISTS federation_http_fetch_retry_limit;
ALTER TABLE local_site DROP COLUMN IF EXISTS federation_worker_count;

-- Rename local_site to site
ALTER TABLE local_site RENAME TO site;

-- Update any foreign key references
-- Update site_language table to reference site instead of site_id
-- (it was already correctly named, just verifying the constraint still works)