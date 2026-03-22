-- Add actor_user_id to notifications so we know WHO triggered the notification
ALTER TABLE notifications ADD COLUMN actor_user_id UUID REFERENCES users(id) ON DELETE CASCADE;

CREATE INDEX idx_notifications_actor ON notifications (actor_user_id);
