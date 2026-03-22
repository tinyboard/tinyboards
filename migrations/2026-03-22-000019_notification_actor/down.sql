DROP INDEX IF EXISTS idx_notifications_actor;
ALTER TABLE notifications DROP COLUMN IF EXISTS actor_user_id;
