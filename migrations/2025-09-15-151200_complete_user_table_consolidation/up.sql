-- Complete the user table consolidation by:
-- 1. Dropping the old person and local_user tables (data already migrated)
-- 2. Renaming admin_purge_person to admin_purge_user for consistency
-- 3. Renaming local_user_language to user_language for consistency

-- Drop person_aggregates table first (has foreign key to person)
DROP TABLE IF EXISTS person_aggregates CASCADE;

-- Drop local_user table (data already migrated to users table)
DROP TABLE IF EXISTS local_user CASCADE;

-- Drop person table (data already migrated to users table)
DROP TABLE IF EXISTS person CASCADE;

-- Rename admin_purge_person to admin_purge_user for consistency
ALTER TABLE admin_purge_person RENAME TO admin_purge_user;

-- Rename local_user_language to user_language for consistency
ALTER TABLE local_user_language RENAME TO user_language;