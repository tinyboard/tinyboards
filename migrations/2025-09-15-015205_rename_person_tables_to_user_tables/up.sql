-- Rename all person_ tables to user_ tables for consistency

-- Rename person_ban to user_ban
ALTER TABLE person_ban RENAME TO user_ban;

-- Rename person_blocks to user_blocks
ALTER TABLE person_blocks RENAME TO user_blocks;

-- Rename person_board_blocks to user_board_blocks
ALTER TABLE person_board_blocks RENAME TO user_board_blocks;

-- Rename person_subscriber to user_subscriber
ALTER TABLE person_subscriber RENAME TO user_subscriber;

-- Rename board_person_bans to board_user_bans
ALTER TABLE board_person_bans RENAME TO board_user_bans;

-- Note: person_aggregates was already replaced with user_aggregates in previous migration