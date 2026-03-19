-- Revert section_config default to 0
ALTER TABLE boards ALTER COLUMN section_config SET DEFAULT 0;
