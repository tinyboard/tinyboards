-- Revert signature column back to URL type
-- Note: This will fail if there's text data that can't be converted to URLs
ALTER TABLE users ALTER COLUMN signature TYPE VARCHAR(255);
