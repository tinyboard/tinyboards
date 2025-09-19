-- Add unique constraints to prevent duplicate votes from same user
-- For post votes: user can only vote once per post
ALTER TABLE post_votes ADD CONSTRAINT post_votes_user_post_unique UNIQUE (user_id, post_id);

-- For comment votes: user can only vote once per comment
ALTER TABLE comment_votes ADD CONSTRAINT comment_votes_user_comment_unique UNIQUE (user_id, comment_id);
