-- Revert language_id back to NOT NULL with default 0
ALTER TABLE posts ALTER COLUMN language_id SET NOT NULL;
ALTER TABLE posts ALTER COLUMN language_id SET DEFAULT 0;

ALTER TABLE comments ALTER COLUMN language_id SET NOT NULL;
ALTER TABLE comments ALTER COLUMN language_id SET DEFAULT 0;
