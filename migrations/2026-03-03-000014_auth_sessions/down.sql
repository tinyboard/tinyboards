ALTER TABLE email_verification DROP COLUMN IF EXISTS verified_at;
ALTER TABLE password_resets DROP COLUMN IF EXISTS used_at;
ALTER TABLE password_resets DROP COLUMN IF EXISTS expires_at;
DROP TABLE IF EXISTS auth_sessions;
