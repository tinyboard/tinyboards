DROP FUNCTION IF EXISTS disable_board_section(INTEGER, INTEGER);
DROP FUNCTION IF EXISTS enable_board_section(INTEGER, INTEGER);
DROP FUNCTION IF EXISTS board_has_section(INTEGER, INTEGER);

ALTER TABLE boards DROP CONSTRAINT IF EXISTS boards_section_config_not_zero;
DROP INDEX IF EXISTS idx_boards_section_config;
ALTER TABLE boards DROP COLUMN IF EXISTS section_config;
