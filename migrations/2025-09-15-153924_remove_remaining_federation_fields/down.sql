-- Restore federation-related fields to site table

-- Add back the site_id column
ALTER TABLE site ADD COLUMN site_id INTEGER NOT NULL DEFAULT 1;

-- Add back federation-related fields
ALTER TABLE site ADD COLUMN actor_name_max_length INTEGER DEFAULT 20 NOT NULL;
ALTER TABLE site ADD COLUMN actor_id TEXT UNIQUE;
ALTER TABLE site ADD COLUMN inbox_url TEXT UNIQUE;
ALTER TABLE site ADD COLUMN last_refreshed_date TIMESTAMP WITHOUT TIME ZONE DEFAULT now() NOT NULL;
ALTER TABLE site ADD COLUMN private_key TEXT;
ALTER TABLE site ADD COLUMN public_key TEXT;
ALTER TABLE site ADD COLUMN instance_id INTEGER;