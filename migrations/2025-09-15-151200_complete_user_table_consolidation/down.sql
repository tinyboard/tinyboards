-- Revert the user table consolidation by recreating the old tables
-- Note: This down migration will lose data since the old tables were consolidated

-- Rename back user_language to local_user_language
ALTER TABLE user_language RENAME TO local_user_language;

-- Rename back admin_purge_user to admin_purge_person
ALTER TABLE admin_purge_user RENAME TO admin_purge_person;

-- Recreate person table (empty)
CREATE TABLE person (
    id SERIAL PRIMARY KEY,
    name VARCHAR(30) NOT NULL UNIQUE,
    display_name VARCHAR(30),
    is_banned BOOLEAN NOT NULL DEFAULT false,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    updated TIMESTAMP,
    avatar TEXT,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    unban_date TIMESTAMP,
    banner TEXT,
    bio TEXT,
    signature TEXT,
    bot_account BOOLEAN NOT NULL DEFAULT false,
    last_refreshed_date TIMESTAMP NOT NULL DEFAULT now(),
    instance_id INTEGER NOT NULL DEFAULT 1,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    instance VARCHAR(256),
    admin_level INTEGER NOT NULL DEFAULT 0,
    profile_background VARCHAR(512),
    avatar_frame VARCHAR(512),
    bio_html VARCHAR(512),
    profile_music VARCHAR(512),
    profile_music_youtube VARCHAR(255),
    board_creation_approved BOOLEAN NOT NULL DEFAULT false
);

-- Recreate local_user table (empty)
CREATE TABLE local_user (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    person_id INTEGER NOT NULL REFERENCES person(id) ON UPDATE CASCADE ON DELETE CASCADE,
    passhash TEXT NOT NULL,
    email TEXT,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    unban_date TIMESTAMP,
    show_nsfw BOOLEAN NOT NULL DEFAULT false,
    show_bots BOOLEAN NOT NULL DEFAULT true,
    theme TEXT NOT NULL DEFAULT 'browser',
    default_sort_type SMALLINT NOT NULL DEFAULT 0,
    default_listing_type SMALLINT NOT NULL DEFAULT 1,
    interface_language TEXT NOT NULL DEFAULT 'browser',
    email_notifications_enabled BOOLEAN NOT NULL DEFAULT false,
    accepted_application BOOLEAN NOT NULL DEFAULT false,
    is_application_accepted BOOLEAN NOT NULL DEFAULT false,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    updated TIMESTAMP,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    admin_level INTEGER NOT NULL DEFAULT 0
);

-- Recreate person_aggregates table (empty)
CREATE TABLE person_aggregates (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON UPDATE CASCADE ON DELETE CASCADE,
    post_count BIGINT NOT NULL DEFAULT 0,
    post_score BIGINT NOT NULL DEFAULT 0,
    comment_count BIGINT NOT NULL DEFAULT 0,
    comment_score BIGINT NOT NULL DEFAULT 0,
    rep BIGINT NOT NULL DEFAULT 0
);