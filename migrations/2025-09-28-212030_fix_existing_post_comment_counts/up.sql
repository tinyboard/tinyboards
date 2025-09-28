-- Fix existing post aggregates comment counts by recalculating from actual comments
UPDATE post_aggregates
SET comments = (
    SELECT COUNT(*)
    FROM comments
    WHERE comments.post_id = post_aggregates.post_id
    AND comments.is_deleted = false
    AND comments.is_removed = false
);

-- Also update newest_comment_time
UPDATE post_aggregates
SET newest_comment_time = (
    SELECT MAX(creation_date)
    FROM comments
    WHERE comments.post_id = post_aggregates.post_id
    AND comments.is_deleted = false
    AND comments.is_removed = false
)
WHERE EXISTS (
    SELECT 1
    FROM comments
    WHERE comments.post_id = post_aggregates.post_id
    AND comments.is_deleted = false
    AND comments.is_removed = false
);
