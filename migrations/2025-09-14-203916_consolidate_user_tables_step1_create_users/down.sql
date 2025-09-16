-- Rollback: Drop the new users tables
DROP INDEX IF EXISTS idx_user_aggregates_user_id;
DROP TABLE IF EXISTS user_aggregates;

DROP INDEX IF EXISTS idx_users_creation_date;
DROP INDEX IF EXISTS idx_users_is_deleted;
DROP INDEX IF EXISTS idx_users_is_banned;
DROP INDEX IF EXISTS idx_users_is_admin;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_name;

DROP TABLE IF EXISTS users;
