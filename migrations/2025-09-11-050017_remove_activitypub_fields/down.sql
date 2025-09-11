-- Add back ActivityPub fields (for rollback, though they won't be used)

-- Add back to person table
ALTER TABLE person ADD COLUMN actor_id TEXT;
ALTER TABLE person ADD COLUMN local BOOLEAN DEFAULT true;
ALTER TABLE person ADD COLUMN inbox_url TEXT;
ALTER TABLE person ADD COLUMN shared_inbox_url TEXT;

-- Add back to boards table
ALTER TABLE boards ADD COLUMN actor_id TEXT;
ALTER TABLE boards ADD COLUMN local BOOLEAN DEFAULT true;
ALTER TABLE boards ADD COLUMN subscribers_url TEXT;
ALTER TABLE boards ADD COLUMN inbox_url TEXT;
ALTER TABLE boards ADD COLUMN shared_inbox_url TEXT;
ALTER TABLE boards ADD COLUMN moderators_url TEXT;
ALTER TABLE boards ADD COLUMN featured_url TEXT;

-- Add back to site table
ALTER TABLE site ADD COLUMN actor_id TEXT;
ALTER TABLE site ADD COLUMN inbox_url TEXT;

-- Add back to comments table
ALTER TABLE comments ADD COLUMN local BOOLEAN DEFAULT true;
ALTER TABLE comments ADD COLUMN ap_id TEXT;

-- Add back to posts table
ALTER TABLE posts ADD COLUMN ap_id TEXT;
ALTER TABLE posts ADD COLUMN local BOOLEAN DEFAULT true;