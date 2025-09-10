-- Remove exclude_from_all column from boards table

ALTER TABLE boards DROP COLUMN IF EXISTS exclude_from_all;