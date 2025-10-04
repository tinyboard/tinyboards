-- Add post_type column to distinguish Feed vs Thread posts
ALTER TABLE posts
ADD COLUMN post_type VARCHAR(10) NOT NULL DEFAULT 'feed';

-- Add indexes for efficient filtering
CREATE INDEX idx_posts_post_type ON posts(post_type);
CREATE INDEX idx_posts_board_post_type_created ON posts(board_id, post_type, creation_date DESC);

-- Ensure all existing posts are marked as 'feed'
UPDATE posts SET post_type = 'feed' WHERE post_type IS NULL OR post_type = '';
