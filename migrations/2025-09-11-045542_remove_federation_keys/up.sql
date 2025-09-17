-- Remove federation keys from all tables since federation is no longer needed
ALTER TABLE person DROP COLUMN IF EXISTS public_key;
ALTER TABLE person DROP COLUMN IF EXISTS private_key;

ALTER TABLE boards DROP COLUMN IF EXISTS public_key;
ALTER TABLE boards DROP COLUMN IF EXISTS private_key;

ALTER TABLE site DROP COLUMN IF EXISTS public_key;
ALTER TABLE site DROP COLUMN IF EXISTS private_key;