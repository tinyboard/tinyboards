-- Replace the bitmask-based sections system with a single board_mode enum.
-- A board is either a "feed" board (links/images/text, upvote-driven) or
-- a "forum" board (threaded discussions).

-- Step 1: Create the enum type
CREATE TYPE board_mode AS ENUM ('feed', 'forum');

-- Step 2: Add the mode column to boards, deriving the value from section_config.
--   section_config bitmask: bit 0 = feed, bit 1 = threads
--   If only threads enabled (& 2 != 0, & 1 = 0) → forum, otherwise → feed.
ALTER TABLE boards ADD COLUMN mode board_mode NOT NULL DEFAULT 'feed';

UPDATE boards SET mode = CASE
    WHEN (section_config & 2) != 0 AND (section_config & 1) = 0 THEN 'forum'::board_mode
    ELSE 'feed'::board_mode
END;

-- Step 3: Drop the old sections columns
ALTER TABLE boards DROP COLUMN section_config;
ALTER TABLE boards DROP COLUMN section_order;
ALTER TABLE boards DROP COLUMN default_section;

-- Step 4: Add default_board_mode to site configuration
ALTER TABLE site ADD COLUMN default_board_mode board_mode NOT NULL DEFAULT 'feed';
