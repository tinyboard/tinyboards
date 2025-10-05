-- This file should undo anything in `up.sql`
ALTER TABLE boards DROP COLUMN section_order;
ALTER TABLE boards DROP COLUMN default_section;
