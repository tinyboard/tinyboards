-- Step 2: Migrate data from person and local_user tables to the new users table
-- This is a safe operation that copies data without deleting original tables

INSERT INTO users (
    id,
    name,
    display_name,
    email,
    passhash,
    email_verified,
    is_banned,
    is_deleted,
    is_admin,
    admin_level,
    unban_date,
    bio,
    bio_html,
    signature,
    avatar,
    banner,
    profile_background,
    avatar_frame,
    profile_music,
    profile_music_youtube,
    show_nsfw,
    show_bots,
    theme,
    default_sort_type,
    default_listing_type,
    interface_language,
    email_notifications_enabled,
    bot_account,
    board_creation_approved,
    accepted_application,
    is_application_accepted,
    creation_date,
    updated
)
SELECT 
    p.id,
    p.name,
    p.display_name,
    COALESCE(lu.email, NULL),
    COALESCE(lu.passhash, ''),  -- Default empty passhash for users without local_user record
    COALESCE(lu.email_verified, false),
    p.is_banned,
    p.is_deleted,
    p.is_admin,
    COALESCE(lu.admin_level, p.admin_level),  -- Prefer local_user admin_level, fallback to person
    p.unban_date,
    p.bio,
    p.bio_html,
    p.signature,
    p.avatar,
    p.banner,
    p.profile_background,
    p.avatar_frame,
    p.profile_music,
    p.profile_music_youtube,
    COALESCE(lu.show_nsfw, false),
    COALESCE(lu.show_bots, false),
    COALESCE(lu.theme, 'browser'),
    COALESCE(lu.default_sort_type, 0),
    COALESCE(lu.default_listing_type, 1),
    COALESCE(lu.interface_language, 'browser'),
    COALESCE(lu.email_notifications_enabled, false),
    p.bot_account,
    p.board_creation_approved,
    COALESCE(lu.accepted_application, false),
    COALESCE(lu.is_application_accepted, false),
    p.creation_date,
    GREATEST(p.updated, lu.updated)  -- Use the most recent update time
FROM person p
LEFT JOIN local_user lu ON p.id = lu.person_id;

-- Migrate person_aggregates to user_aggregates
INSERT INTO user_aggregates (
    id,
    user_id,
    post_count,
    post_score,
    comment_count,
    comment_score,
    rep
)
SELECT 
    id,
    person_id,  -- This becomes user_id
    post_count,
    post_score,
    comment_count,
    comment_score,
    rep
FROM person_aggregates;

-- Update the sequence values to prevent ID conflicts
SELECT setval('users_id_seq', (SELECT COALESCE(MAX(id), 1) FROM users));
SELECT setval('user_aggregates_id_seq', (SELECT COALESCE(MAX(id), 1) FROM user_aggregates));
