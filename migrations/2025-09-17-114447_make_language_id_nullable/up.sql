-- Make language_id nullable for posts and comments to simplify creation
ALTER TABLE posts ALTER COLUMN language_id DROP NOT NULL;
ALTER TABLE posts ALTER COLUMN language_id SET DEFAULT NULL;

ALTER TABLE comments ALTER COLUMN language_id DROP NOT NULL;
ALTER TABLE comments ALTER COLUMN language_id SET DEFAULT NULL;
