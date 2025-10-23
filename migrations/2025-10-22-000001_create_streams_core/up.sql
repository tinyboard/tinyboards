-- Migration 1: Core Streams Tables
-- This migration creates the foundational streams system with support for personalized content feeds

-- ==================== STREAMS TABLE ====================
-- Main table for storing user-created content streams (custom feeds)
CREATE TABLE streams (
    id SERIAL PRIMARY KEY,
    creator_id INTEGER NOT NULL,

    -- Identity and display
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    icon VARCHAR(255),
    color VARCHAR(25) DEFAULT '#3b82f6', -- Default blue color for navbar

    -- Privacy and discovery settings
    is_public BOOLEAN NOT NULL DEFAULT false,
    is_discoverable BOOLEAN NOT NULL DEFAULT false,
    share_token VARCHAR(64) UNIQUE, -- For sharing private streams via link

    -- Feed configuration
    sort_type VARCHAR(20) NOT NULL DEFAULT 'Hot', -- Hot, New, Top, TopDay, TopWeek, etc.
    time_range VARCHAR(20), -- For Top sorting: Day, Week, Month, Year, All
    show_nsfw BOOLEAN NOT NULL DEFAULT false,
    max_posts_per_board INTEGER, -- Optional limit to prevent single board dominance

    -- Metadata
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP,
    last_viewed_at TIMESTAMP, -- Track when stream was last accessed by creator

    -- Foreign key
    CONSTRAINT fk_streams_creator
        FOREIGN KEY (creator_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- Create indexes for efficient querying
CREATE INDEX idx_streams_creator_id ON streams(creator_id);
CREATE INDEX idx_streams_slug ON streams(slug);
CREATE INDEX idx_streams_share_token ON streams(share_token) WHERE share_token IS NOT NULL;
CREATE INDEX idx_streams_is_public ON streams(is_public);
CREATE INDEX idx_streams_is_discoverable ON streams(is_discoverable);
CREATE INDEX idx_streams_created_at ON streams(created_at DESC);

-- Unique constraint: each user can only have one stream with a given slug
CREATE UNIQUE INDEX idx_streams_creator_slug ON streams(creator_id, slug);

-- Comments for documentation
COMMENT ON TABLE streams IS 'User-created content streams (custom feeds) that aggregate posts based on flair and/or board subscriptions';
COMMENT ON COLUMN streams.share_token IS 'Unique token for sharing private streams via URL. Generated only for shared private streams.';
COMMENT ON COLUMN streams.max_posts_per_board IS 'Optional limit to prevent a single board from dominating the stream feed';
COMMENT ON COLUMN streams.sort_type IS 'Default sorting for stream: Hot, New, Top, TopDay, TopWeek, TopMonth, TopYear, TopAll';
