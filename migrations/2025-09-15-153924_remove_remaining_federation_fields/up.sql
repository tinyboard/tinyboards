-- Remove remaining federation-related fields from site table

-- Remove the site_id column since this table is now the main site table
ALTER TABLE site DROP COLUMN IF EXISTS site_id;

-- Remove federation-related fields
ALTER TABLE site DROP COLUMN IF EXISTS actor_name_max_length;

-- Remove any remaining federation fields that might exist
-- (these may not exist but adding IF EXISTS for safety)
ALTER TABLE site DROP COLUMN IF EXISTS actor_id;
ALTER TABLE site DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE site DROP COLUMN IF EXISTS last_refreshed_date;
ALTER TABLE site DROP COLUMN IF EXISTS private_key;
ALTER TABLE site DROP COLUMN IF EXISTS public_key;
ALTER TABLE site DROP COLUMN IF EXISTS instance_id;