-- Revert board_mode changes back to section_config bitmask system.

ALTER TABLE site DROP COLUMN default_board_mode;

ALTER TABLE boards ADD COLUMN section_config INTEGER NOT NULL DEFAULT 3;
ALTER TABLE boards ADD COLUMN section_order TEXT;
ALTER TABLE boards ADD COLUMN default_section TEXT;

UPDATE boards SET section_config = CASE
    WHEN mode = 'forum' THEN 2
    ELSE 1
END;

ALTER TABLE boards DROP COLUMN mode;

DROP TYPE board_mode;
