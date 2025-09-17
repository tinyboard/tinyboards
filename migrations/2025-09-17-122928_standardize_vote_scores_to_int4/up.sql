-- Standardize vote score columns from smallint to integer for consistency
-- This allows using i32 throughout the Rust codebase instead of i16

ALTER TABLE post_votes ALTER COLUMN score TYPE integer;
ALTER TABLE comment_votes ALTER COLUMN score TYPE integer;
