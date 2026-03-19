-- Fix section_config default: enable both feed (bit 0) and threads (bit 1) by default.
-- Previous default of 0 meant neither feed nor threads were enabled, causing post creation to fail.
ALTER TABLE boards ALTER COLUMN section_config SET DEFAULT 3;

-- Update existing boards that have section_config = 0 (nothing enabled) to enable both sections.
UPDATE boards SET section_config = 3 WHERE section_config = 0;
