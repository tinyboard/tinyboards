-- Add last_seen column to users table
ALTER TABLE users ADD COLUMN last_seen TIMESTAMP DEFAULT NOW() NOT NULL;

-- Create index for efficient querying of online users
CREATE INDEX idx_users_last_seen ON users(last_seen DESC);

-- Update existing users to have current timestamp
UPDATE users SET last_seen = NOW();
