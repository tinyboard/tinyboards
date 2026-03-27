-- Add custom CSS support to site and boards tables.
-- Site custom CSS applies globally; board custom CSS overrides within that board's pages.

ALTER TABLE site
    ADD COLUMN custom_css TEXT,
    ADD COLUMN custom_css_enabled BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE boards
    ADD COLUMN custom_css TEXT;
