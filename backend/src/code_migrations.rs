use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::*,
    models::user::user::{User, UserInsertForm},
    schema::{site, users},
    utils::DbPool,
};
use tinyboards_utils::{
    error::TinyBoardsError, settings::structs::Settings,
};
use tracing::info;
use uuid::Uuid;

pub async fn run_advanced_migrations(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    initialize_core_database_records(pool).await?;
    initialize_site_and_admin_user(pool, settings).await?;

    Ok(())
}

/// Initialize core database records that are essential for the application to function
async fn initialize_core_database_records(pool: &DbPool) -> Result<(), TinyBoardsError> {
    info!("Running initialize_core_database_records");

    let mut conn = pool.get().await?;

    // Insert default English language if it doesn't exist
    sql_query("INSERT INTO languages (code, name) VALUES ('en', 'English') ON CONFLICT (code) DO NOTHING")
        .execute(&mut conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert default language"))?;

    // Insert JWT secret if it doesn't exist
    sql_query("INSERT INTO secrets (id, jwt_secret) SELECT gen_random_uuid(), encode(gen_random_bytes(32), 'hex') WHERE NOT EXISTS (SELECT 1 FROM secrets)")
        .execute(&mut conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert JWT secret"))?;

    info!("Core database records initialized");
    Ok(())
}

/// This ensures the site is initialized
async fn initialize_site_and_admin_user(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    info!("Running initialize_site_and_admin_user");

    let mut conn = pool.get().await?;

    // Check if site already exists
    let site_exists: bool = site::table
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map(|c| c > 0)
        .unwrap_or(false);

    if site_exists {
        info!("Site already initialized, skipping setup");
        return Ok(());
    }

    info!("No Site found, initializing TinyBoards!");

    if let Some(setup) = &settings.setup {
        // Hash password using argon2
        use argon2::Argon2;
        use argon2::password_hash::{SaltString, PasswordHasher};
        use argon2::password_hash::rand_core::OsRng;

        let salt = SaltString::generate(&mut OsRng);
        let passhash = Argon2::default()
            .hash_password(setup.admin_password.as_bytes(), &salt)
            .map_err(|e| TinyBoardsError::from_message(500, &format!("Password hash error: {}", e)))?
            .to_string();

        // Create admin user
        let user_form = UserInsertForm {
            name: setup.admin_username.clone(),
            display_name: Some(setup.admin_username.clone()),
            email: setup.admin_email.clone(),
            passhash,
            is_email_verified: true,
            is_banned: false,
            is_admin: true,
            admin_level: 256,
            is_bot_account: false,
            is_board_creation_approved: true,
            is_application_accepted: true,
            unban_date: None,
            bio: None,
            bio_html: None,
            signature: None,
            avatar: None,
            banner: None,
            profile_background: None,
            avatar_frame: None,
            profile_music: None,
            profile_music_youtube: None,
            show_nsfw: false,
            show_bots: true,
            theme: "default".to_string(),
            default_sort_type: DbSortType::Hot,
            default_listing_type: DbListingType::All,
            interface_language: "en".to_string(),
            is_email_notifications_enabled: false,
            editor_mode: DbEditorMode::RichText,
        };

        let inserted_admin: User = match diesel::insert_into(users::table)
            .values(&user_form)
            .get_result::<User>(&mut conn)
            .await
        {
            Ok(user) => {
                info!("Admin user '{}' created successfully", user.name);
                user
            }
            Err(e) => {
                info!("Admin user might already exist: {:?}", e);
                users::table
                    .filter(users::name.eq(&setup.admin_username))
                    .first(&mut conn)
                    .await
                    .map_err(|_| TinyBoardsError::from_error_message(e, 500, "Failed to create or find admin user"))?
            }
        };

        // Create default board using raw SQL (BoardInsertForm has many required fields)
        let board_name = &setup.default_board_name;
        let board_desc = setup.default_board_description.as_deref().unwrap_or("");
        sql_query(format!(
            "INSERT INTO boards (id, name, title, description, primary_color, secondary_color, hover_color) \
             VALUES (gen_random_uuid(), '{}', '{}', '{}', '#3b82f6', '#1e40af', '#2563eb') \
             ON CONFLICT (name) DO NOTHING",
            board_name, board_name, board_desc
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            info!("Default board might already exist: {:?}", e);
            TinyBoardsError::from_error_message(e, 500, "Failed to create default board")
        })
        .ok();

        // Add the admin as the board owner (rank 0) in board_moderators
        sql_query(format!(
            "INSERT INTO board_moderators (id, board_id, user_id, permissions, rank, is_invite_accepted, invite_accepted_at) \
             SELECT gen_random_uuid(), b.id, '{}', 2147483647, 0, true, now() \
             FROM boards b WHERE b.name = '{}' \
             ON CONFLICT (board_id, user_id) DO NOTHING",
            inserted_admin.id, board_name
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            info!("Admin board moderator entry might already exist: {:?}", e);
            TinyBoardsError::from_error_message(e, 500, "Failed to add admin as board owner")
        })
        .ok();

        info!("Default board '{}' initialized", board_name);

        // Create site
        let site_name = settings
            .setup
            .clone()
            .map(|s| s.site_name)
            .unwrap_or_else(|| "New TinyBoards Site".to_string());

        sql_query(format!(
            "INSERT INTO site (id, name, is_site_setup, registration_mode, enable_downvotes, enable_nsfw, \
             boards_enabled, board_creation_admin_only, board_creation_mode, \
             primary_color, secondary_color, hover_color, default_theme, \
             emoji_enabled, emoji_max_file_size_mb, board_emojis_enabled, \
             captcha_enabled, captcha_difficulty) \
             VALUES (gen_random_uuid(), '{}', true, 'open', true, true, \
             true, false, 'AdminOnly', \
             '#3b82f6', '#1e40af', '#2563eb', 'default', \
             true, 5, true, \
             false, 'medium')",
            site_name
        ))
        .execute(&mut conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create site"))?;

        // Initialize site aggregates
        sql_query("INSERT INTO site_aggregates (id, site_id) SELECT gen_random_uuid(), id FROM site LIMIT 1 ON CONFLICT DO NOTHING")
            .execute(&mut conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert site aggregates"))?;

        // Initialize default rate limits for the site
        sql_query("INSERT INTO rate_limits (id, site_id) SELECT gen_random_uuid(), id FROM site LIMIT 1 ON CONFLICT DO NOTHING")
            .execute(&mut conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert rate limits"))?;

        // Seed welcome posts so the home page isn't empty on first launch
        seed_welcome_posts(pool, &inserted_admin.id, board_name).await;

        info!("Site '{}' and admin user successfully initialized!", site_name);

        Ok(())
    } else {
        info!("No setup configuration found in settings, skipping initialization");
        Ok(())
    }
}

/// Creates a few welcome posts in the default board so new instances don't look empty
async fn seed_welcome_posts(
    pool: &DbPool,
    admin_id: &Uuid,
    board_name: &str,
) {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            info!("Could not get DB connection for seed posts: {:?}", e);
            return;
        }
    };
    let conn = &mut *conn;
    let welcome_posts = [
        (
            "Welcome to TinyBoards!",
            "This is your new community platform. TinyBoards is a self-hosted social media platform where you can create boards, share posts, and build communities.\n\nFeel free to explore, create new boards, and invite others to join!",
            "<p>This is your new community platform. TinyBoards is a self-hosted social media platform where you can create boards, share posts, and build communities.</p>\n<p>Feel free to explore, create new boards, and invite others to join!</p>",
            true, // pin this one
        ),
        (
            "Getting Started Guide",
            "Here are some tips to get you started:\n\n1. **Create a board** — Boards are topic-based communities. Visit the Boards page to create your first one.\n2. **Customize your profile** — Click your username to update your avatar, bio, and display name.\n3. **Invite your friends** — Share the link to your instance and grow your community.\n4. **Check the admin panel** — If you're the admin, head to the settings to configure your site.",
            "<p>Here are some tips to get you started:</p>\n<ol>\n<li><strong>Create a board</strong> — Boards are topic-based communities. Visit the Boards page to create your first one.</li>\n<li><strong>Customize your profile</strong> — Click your username to update your avatar, bio, and display name.</li>\n<li><strong>Invite your friends</strong> — Share the link to your instance and grow your community.</li>\n<li><strong>Check the admin panel</strong> — If you're the admin, head to the settings to configure your site.</li>\n</ol>",
            false,
        ),
        (
            "What can you do on TinyBoards?",
            "TinyBoards supports:\n\n- **Text posts and link sharing** — Share articles, discussions, and links\n- **Nested comments** — Have threaded conversations on any post\n- **Voting** — Upvote and downvote posts and comments\n- **Multiple boards** — Organize content into topic-specific communities\n- **User profiles** — Customize your identity with avatars and bios\n- **Moderation tools** — Keep your communities healthy with built-in mod features",
            "<p>TinyBoards supports:</p>\n<ul>\n<li><strong>Text posts and link sharing</strong> — Share articles, discussions, and links</li>\n<li><strong>Nested comments</strong> — Have threaded conversations on any post</li>\n<li><strong>Voting</strong> — Upvote and downvote posts and comments</li>\n<li><strong>Multiple boards</strong> — Organize content into topic-specific communities</li>\n<li><strong>User profiles</strong> — Customize your identity with avatars and bios</li>\n<li><strong>Moderation tools</strong> — Keep your communities healthy with built-in mod features</li>\n</ul>",
            false,
        ),
    ];

    for (title, body, body_html, is_featured) in welcome_posts {
        let result = sql_query(format!(
            "INSERT INTO posts (id, title, body, body_html, creator_id, board_id, is_featured_board) \
             SELECT gen_random_uuid(), $1, $2, $3, '{admin_id}', b.id, {featured} \
             FROM boards b WHERE b.name = '{board_name}' \
             AND NOT EXISTS (SELECT 1 FROM posts p WHERE p.title = $1 AND p.board_id = b.id)",
            admin_id = admin_id,
            featured = is_featured,
            board_name = board_name,
        ));

        // Use bind params for the text content to avoid SQL injection issues
        use diesel::sql_types::Text;
        match result
            .bind::<Text, _>(title)
            .bind::<Text, _>(body)
            .bind::<Text, _>(body_html)
            .execute(conn)
            .await
        {
            Ok(_) => info!("Seed post '{}' created", title),
            Err(e) => info!("Seed post '{}' may already exist: {:?}", title, e),
        }
    }

    // Auto-upvote the seed posts by the admin
    let _ = sql_query(format!(
        "INSERT INTO post_votes (id, post_id, user_id, score) \
         SELECT gen_random_uuid(), p.id, '{admin_id}', 1 \
         FROM posts p \
         JOIN boards b ON b.id = p.board_id \
         WHERE b.name = '{board_name}' AND p.creator_id = '{admin_id}' \
         AND NOT EXISTS (SELECT 1 FROM post_votes pv WHERE pv.post_id = p.id AND pv.user_id = '{admin_id}')",
        admin_id = admin_id,
        board_name = board_name,
    ))
    .execute(conn)
    .await;

    info!("Welcome posts seeded successfully");
}
