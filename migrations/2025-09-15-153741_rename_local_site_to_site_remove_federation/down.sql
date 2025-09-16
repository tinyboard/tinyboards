-- Reverse the renaming and restore federation fields

-- Rename site back to local_site
ALTER TABLE site RENAME TO local_site;

-- Add back federation-related columns
ALTER TABLE local_site ADD COLUMN federation_debug BOOLEAN DEFAULT false NOT NULL;
ALTER TABLE local_site ADD COLUMN federation_strict_allowlist BOOLEAN DEFAULT false NOT NULL;
ALTER TABLE local_site ADD COLUMN federation_http_fetch_retry_limit INTEGER DEFAULT 25 NOT NULL;
ALTER TABLE local_site ADD COLUMN federation_worker_count INTEGER DEFAULT 64 NOT NULL;

-- Recreate the old site table structure (minimal version for reference)
CREATE TABLE site (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    sidebar TEXT,
    creator_id INTEGER NOT NULL,
    creation_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITHOUT TIME ZONE,
    icon TEXT,
    banner TEXT,
    description TEXT,
    actor_id TEXT UNIQUE,
    last_refreshed_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    inbox_url TEXT UNIQUE,
    private_key TEXT,
    public_key TEXT NOT NULL,
    instance_id INTEGER NOT NULL
);