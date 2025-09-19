-- Create table for hidden posts
CREATE TABLE post_hidden (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    creation_date TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(post_id, user_id)
);

-- Create index for efficient queries
CREATE INDEX idx_post_hidden_user_id ON post_hidden(user_id);
CREATE INDEX idx_post_hidden_post_id ON post_hidden(post_id);