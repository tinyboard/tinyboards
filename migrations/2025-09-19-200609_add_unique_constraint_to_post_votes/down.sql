-- Remove the unique constraints added in up.sql
ALTER TABLE post_votes DROP CONSTRAINT post_votes_user_post_unique;
ALTER TABLE comment_votes DROP CONSTRAINT comment_votes_user_comment_unique;
