-- Revert vote score columns back to smallint
ALTER TABLE post_votes ALTER COLUMN score TYPE smallint;
ALTER TABLE comment_votes ALTER COLUMN score TYPE smallint;
