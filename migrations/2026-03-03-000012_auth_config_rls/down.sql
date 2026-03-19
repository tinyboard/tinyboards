-- Drop RLS policies
DROP POLICY IF EXISTS user_blocks_owner_policy ON user_blocks;
DROP POLICY IF EXISTS post_hidden_owner_policy ON post_hidden;
DROP POLICY IF EXISTS comment_saved_owner_policy ON comment_saved;
DROP POLICY IF EXISTS post_saved_owner_policy ON post_saved;
DROP POLICY IF EXISTS notification_settings_owner_policy ON notification_settings;
DROP POLICY IF EXISTS notification_owner_policy ON notifications;
DROP POLICY IF EXISTS pm_owner_policy ON private_messages;

-- Disable RLS
ALTER TABLE user_blocks DISABLE ROW LEVEL SECURITY;
ALTER TABLE post_hidden DISABLE ROW LEVEL SECURITY;
ALTER TABLE comment_saved DISABLE ROW LEVEL SECURITY;
ALTER TABLE post_saved DISABLE ROW LEVEL SECURITY;
ALTER TABLE notification_settings DISABLE ROW LEVEL SECURITY;
ALTER TABLE notifications DISABLE ROW LEVEL SECURITY;
ALTER TABLE private_messages DISABLE ROW LEVEL SECURITY;

-- Drop tables
DROP TABLE IF EXISTS rate_limits CASCADE;
DROP TABLE IF EXISTS site_invites CASCADE;
DROP TABLE IF EXISTS registration_applications CASCADE;
DROP TABLE IF EXISTS email_verification CASCADE;
DROP TABLE IF EXISTS password_resets CASCADE;
DROP TABLE IF EXISTS secrets CASCADE;
