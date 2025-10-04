-- Add quoted_comment_id for quote-based replies in threads
ALTER TABLE comments ADD COLUMN quoted_comment_id INT4;

-- Add foreign key constraint
ALTER TABLE comments ADD CONSTRAINT fk_quoted_comment
  FOREIGN KEY (quoted_comment_id) REFERENCES comments(id) ON DELETE SET NULL;

-- Add index for efficient lookups
CREATE INDEX idx_comments_quoted_comment_id ON comments(quoted_comment_id);
