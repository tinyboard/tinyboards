-- Step 1: Create the consolidated users table 
-- This is a safe operation that doesn't modify existing data

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    
    -- Core Identity (from person + local_user)
    name VARCHAR(30) NOT NULL UNIQUE,
    display_name VARCHAR(30),
    email TEXT,
    
    -- Authentication (from local_user)  
    passhash TEXT NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    
    -- Status & Permissions (merged from both)
    is_banned BOOLEAN NOT NULL DEFAULT false,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    admin_level INTEGER NOT NULL DEFAULT 0,
    unban_date TIMESTAMP,
    
    -- Profile & Display (from person)
    bio TEXT,
    bio_html TEXT,
    signature TEXT,
    avatar TEXT,
    banner TEXT,
    profile_background TEXT,
    avatar_frame TEXT,
    profile_music TEXT,
    profile_music_youtube TEXT,
    
    -- User Preferences (from local_user)
    show_nsfw BOOLEAN NOT NULL DEFAULT false,
    show_bots BOOLEAN NOT NULL DEFAULT false,
    theme TEXT NOT NULL DEFAULT 'browser',
    default_sort_type SMALLINT NOT NULL DEFAULT 0,
    default_listing_type SMALLINT NOT NULL DEFAULT 1,
    interface_language TEXT NOT NULL DEFAULT 'browser',
    email_notifications_enabled BOOLEAN NOT NULL DEFAULT false,
    
    -- Bot & Special Accounts (from person)
    bot_account BOOLEAN NOT NULL DEFAULT false,
    board_creation_approved BOOLEAN NOT NULL DEFAULT false,
    
    -- Application System (from local_user)
    accepted_application BOOLEAN NOT NULL DEFAULT false,
    is_application_accepted BOOLEAN NOT NULL DEFAULT false,
    
    -- Timestamps
    creation_date TIMESTAMP NOT NULL DEFAULT NOW(),
    updated TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX idx_users_name ON users(name);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_admin ON users(is_admin);
CREATE INDEX idx_users_is_banned ON users(is_banned);
CREATE INDEX idx_users_is_deleted ON users(is_deleted);
CREATE INDEX idx_users_creation_date ON users(creation_date);

-- Create user_aggregates table (replacing person_aggregates)
CREATE TABLE user_aggregates (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_count BIGINT NOT NULL DEFAULT 0,
    post_score BIGINT NOT NULL DEFAULT 0,
    comment_count BIGINT NOT NULL DEFAULT 0,
    comment_score BIGINT NOT NULL DEFAULT 0,
    rep BIGINT NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX idx_user_aggregates_user_id ON user_aggregates(user_id);
