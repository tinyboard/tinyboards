-- Rollback: Clear the migrated data from users tables
-- Note: This doesn't restore the original data, just clears the new tables
TRUNCATE TABLE user_aggregates CASCADE;
TRUNCATE TABLE users CASCADE;
