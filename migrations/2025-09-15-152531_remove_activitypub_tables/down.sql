-- Restore ActivityPub federation related tables

-- Add federation_enabled field back to local_site table
ALTER TABLE local_site ADD COLUMN federation_enabled BOOLEAN DEFAULT false NOT NULL;

-- Recreate instance table
CREATE TABLE instance (
    id SERIAL PRIMARY KEY,
    domain VARCHAR NOT NULL UNIQUE,
    creation_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITHOUT TIME ZONE
);

-- Recreate activity table
CREATE TABLE activity (
    id SERIAL PRIMARY KEY,
    ap_id VARCHAR NOT NULL UNIQUE,
    data JSONB NOT NULL,
    local BOOLEAN NOT NULL DEFAULT true,
    sensitive BOOLEAN NOT NULL DEFAULT false,
    creation_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITHOUT TIME ZONE
);

-- Recreate federation tables
CREATE TABLE federation_allowlist (
    id SERIAL PRIMARY KEY,
    instance_id INTEGER NOT NULL REFERENCES instance(id) ON DELETE CASCADE,
    creation_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITHOUT TIME ZONE
);

CREATE TABLE federation_blocklist (
    id SERIAL PRIMARY KEY,
    instance_id INTEGER NOT NULL REFERENCES instance(id) ON DELETE CASCADE,
    creation_date TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT now(),
    updated TIMESTAMP WITHOUT TIME ZONE
);