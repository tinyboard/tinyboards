-- Revert user_ table names back to person_ tables

-- Revert user_ban to person_ban
ALTER TABLE user_ban RENAME TO person_ban;

-- Revert user_blocks to person_blocks
ALTER TABLE user_blocks RENAME TO person_blocks;

-- Revert user_board_blocks to person_board_blocks
ALTER TABLE user_board_blocks RENAME TO person_board_blocks;

-- Revert user_subscriber to person_subscriber
ALTER TABLE user_subscriber RENAME TO person_subscriber;

-- Revert board_user_bans to board_person_bans
ALTER TABLE board_user_bans RENAME TO board_person_bans;