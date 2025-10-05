-- Add section order and default section to boards table
ALTER TABLE boards ADD COLUMN section_order TEXT DEFAULT 'feed,threads';
ALTER TABLE boards ADD COLUMN default_section TEXT DEFAULT 'feed';
