-- Reactions table for thread posts and comments
CREATE TABLE reactions (
    id SERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id INT4 REFERENCES posts(id) ON DELETE CASCADE,
    comment_id INT4 REFERENCES comments(id) ON DELETE CASCADE,
    emoji VARCHAR(100) NOT NULL,
    score INT4 NOT NULL DEFAULT 0, -- -1 (negative), 0 (neutral), or 1 (positive)
    creation_date TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Ensure user can only react once with same emoji to same content
    CONSTRAINT unique_user_post_emoji UNIQUE (user_id, post_id, emoji),
    CONSTRAINT unique_user_comment_emoji UNIQUE (user_id, comment_id, emoji),

    -- Must react to either a post or comment, not both
    CONSTRAINT check_post_or_comment CHECK (
        (post_id IS NOT NULL AND comment_id IS NULL) OR
        (post_id IS NULL AND comment_id IS NOT NULL)
    ),

    -- Score must be -1, 0, or 1
    CONSTRAINT check_score_range CHECK (score IN (-1, 0, 1))
);

-- Indexes for efficient lookups
CREATE INDEX idx_reactions_post_id ON reactions(post_id);
CREATE INDEX idx_reactions_comment_id ON reactions(comment_id);
CREATE INDEX idx_reactions_user_id ON reactions(user_id);
CREATE INDEX idx_reactions_emoji ON reactions(emoji);

-- Reaction aggregates for performance (counts only)
CREATE TABLE reaction_aggregates (
    id SERIAL PRIMARY KEY,
    post_id INT4 REFERENCES posts(id) ON DELETE CASCADE,
    comment_id INT4 REFERENCES comments(id) ON DELETE CASCADE,
    emoji VARCHAR(100) NOT NULL,
    count INT4 NOT NULL DEFAULT 0,

    -- Unique constraint for emoji per content
    CONSTRAINT unique_post_emoji UNIQUE (post_id, emoji),
    CONSTRAINT unique_comment_emoji UNIQUE (comment_id, emoji),

    -- Must be for either a post or comment, not both
    CONSTRAINT check_agg_post_or_comment CHECK (
        (post_id IS NOT NULL AND comment_id IS NULL) OR
        (post_id IS NULL AND comment_id IS NOT NULL)
    )
);

-- Indexes for efficient lookups
CREATE INDEX idx_reaction_aggregates_post_id ON reaction_aggregates(post_id);
CREATE INDEX idx_reaction_aggregates_comment_id ON reaction_aggregates(comment_id);

-- Trigger function to update reaction aggregates AND post/comment scores
CREATE OR REPLACE FUNCTION update_reaction_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Increment count for this emoji
        IF NEW.post_id IS NOT NULL THEN
            INSERT INTO reaction_aggregates (post_id, emoji, count)
            VALUES (NEW.post_id, NEW.emoji, 1)
            ON CONFLICT (post_id, emoji)
            DO UPDATE SET count = reaction_aggregates.count + 1;

            -- Update post score if reaction has weight
            IF NEW.score != 0 THEN
                UPDATE post_aggregates
                SET score = score + NEW.score
                WHERE post_id = NEW.post_id;
            END IF;
        ELSIF NEW.comment_id IS NOT NULL THEN
            INSERT INTO reaction_aggregates (comment_id, emoji, count)
            VALUES (NEW.comment_id, NEW.emoji, 1)
            ON CONFLICT (comment_id, emoji)
            DO UPDATE SET count = reaction_aggregates.count + 1;

            -- Update comment score if reaction has weight
            IF NEW.score != 0 THEN
                UPDATE comment_aggregates
                SET score = score + NEW.score
                WHERE comment_id = NEW.comment_id;
            END IF;
        END IF;
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Decrement count for this emoji
        IF OLD.post_id IS NOT NULL THEN
            UPDATE reaction_aggregates
            SET count = count - 1
            WHERE post_id = OLD.post_id AND emoji = OLD.emoji;

            -- Remove aggregate if count reaches 0
            DELETE FROM reaction_aggregates
            WHERE post_id = OLD.post_id AND emoji = OLD.emoji AND count <= 0;

            -- Update post score if reaction had weight
            IF OLD.score != 0 THEN
                UPDATE post_aggregates
                SET score = score - OLD.score
                WHERE post_id = OLD.post_id;
            END IF;
        ELSIF OLD.comment_id IS NOT NULL THEN
            UPDATE reaction_aggregates
            SET count = count - 1
            WHERE comment_id = OLD.comment_id AND emoji = OLD.emoji;

            -- Remove aggregate if count reaches 0
            DELETE FROM reaction_aggregates
            WHERE comment_id = OLD.comment_id AND emoji = OLD.emoji AND count <= 0;

            -- Update comment score if reaction had weight
            IF OLD.score != 0 THEN
                UPDATE comment_aggregates
                SET score = score - OLD.score
                WHERE comment_id = OLD.comment_id;
            END IF;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger
CREATE TRIGGER reaction_aggregates_trigger
    AFTER INSERT OR DELETE ON reactions
    FOR EACH ROW
    EXECUTE FUNCTION update_reaction_aggregates();

-- Board reaction settings (which emojis are allowed and their weights)
CREATE TABLE board_reaction_settings (
    id SERIAL PRIMARY KEY,
    board_id INT4 NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    -- JSONB format: {"👍": 1, "❤️": 1, "😂": 0, "😮": 0, "😢": 0, "👎": -1}
    -- Only values -1, 0, or 1 are allowed
    emoji_weights JSONB NOT NULL DEFAULT '{"👍": 1, "❤️": 1, "😂": 0, "😮": 0, "😢": 0, "👎": -1}'::JSONB,
    reactions_enabled BOOLEAN NOT NULL DEFAULT TRUE,

    CONSTRAINT unique_board_reaction_settings UNIQUE (board_id)
);

-- Create default settings for existing boards
INSERT INTO board_reaction_settings (board_id, reactions_enabled)
SELECT id, TRUE FROM boards;
