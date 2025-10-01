-- Remove hardcoded default avatar URLs from existing users
-- This allows the frontend Avatar component to handle fallback to site's default_avatar or SVG icon
UPDATE users
SET avatar = NULL
WHERE avatar LIKE '%/media/default_pfp.png';
