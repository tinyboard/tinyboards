-- Add distinguished_as column to posts and comments.
-- NULL = not distinguished, 'admin' = speaking as admin, 'mod' = speaking as moderator.
ALTER TABLE posts ADD COLUMN distinguished_as VARCHAR(10) DEFAULT NULL;
ALTER TABLE comments ADD COLUMN distinguished_as VARCHAR(10) DEFAULT NULL;
