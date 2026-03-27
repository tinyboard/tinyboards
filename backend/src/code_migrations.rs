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

/// Creates a welcome post in the default board so new instances don't look empty
async fn seed_welcome_posts(
    pool: &DbPool,
    admin_id: &Uuid,
    board_name: &str,
) {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            info!("Could not get DB connection for seed post: {:?}", e);
            return;
        }
    };
    let conn = &mut *conn;

    let title = "Welcome to TinyBoards — Admin Setup Guide";

    let body = r#"Welcome to your new TinyBoards instance! This pinned post walks you through everything you need to configure before opening your site to users. You can edit or delete this post at any time.

---

## 1. Configure Site Settings

Head to **/admin/settings** to set up the basics:

- **Site name and description** — appears in the header and search results
- **Registration mode** — choose between open registration, invite-only, application-required, or closed
- **Enable/disable downvotes** — decide whether your community uses downvotes
- **NSFW content** — toggle whether NSFW content is allowed site-wide

## 2. Customize Appearance

Visit **/admin/appearance** to brand your site:

- **Site icon and banner** — upload your logo and header image
- **Theme colors** — set primary, secondary, and hover colors
- **Default theme** — choose which of the 6 built-in themes new users see
- **Custom CSS** — add your own styles at **/admin/css**

## 3. Set Up Boards

Boards are the topic-based communities where content lives. You can:

- **Create new boards** from the Boards directory page
- **Configure each board** with its own icon, banner, description, and rules
- **Assign moderators** to help manage individual boards
- **Control board creation** — in site settings, decide if all users or only admins can create boards

## 4. Security Checklist

Before going live, make sure you've handled these:

- [ ] Change the default admin password to something strong
- [ ] Set a unique `salt_suffix` in your `tinyboards.hjson` config file
- [ ] Enable TLS (HTTPS) — set `tls_enabled: true` once your reverse proxy handles SSL
- [ ] Update `cors.allowed_origins` to match your actual domain
- [ ] Review rate limits in your config file and adjust for your expected traffic
- [ ] Consider enabling CAPTCHA for registration at **/admin/security**

## 5. Content & Moderation

Set up your moderation workflow:

- **Content filtering** — configure word filters and content rules at **/admin/filtering**
- **Reports queue** — review user-submitted reports at **/admin/reports**
- **Mod queue** — review flagged content at **/admin/queue**
- **Site bans** — manage site-wide bans at **/admin/bans**

## 6. Optional: Email (SMTP)

Email is needed for password resets and email verification. Add an `email` block to your `tinyboards.hjson`:

```
email: {
  smtp_server: "smtp.example.com:587"
  smtp_login: "your-login"
  smtp_password: "your-password"
  smtp_from_address: "noreply@yourdomain.com"
  tls_type: "starttls"
}
```

Until email is configured, password reset and email verification will not function.

## 7. Optional: Cloud Storage

By default, uploads are stored on the local filesystem. To use S3-compatible storage (AWS S3, MinIO, DigitalOcean Spaces, Cloudflare R2), Azure Blob Storage, or Google Cloud Storage, update the `storage` block in your `tinyboards.hjson`.

## 8. Inviting Users

Depending on your registration mode:

- **Open** — share your site URL and anyone can register
- **Invite-only** — generate invite codes at **/admin/invites** and distribute them
- **Application-required** — users submit applications; review them at **/admin/applications**

## 9. Ongoing Administration

Useful admin pages to bookmark:

- **/admin** — dashboard overview with site statistics
- **/admin/users** — manage users, roles, and permissions
- **/admin/admins** — add or remove site administrators
- **/admin/emojis** — add custom emoji for your community

---

Once you've completed setup, feel free to unpin or delete this post. Happy community building!"#;

    let body_html = r#"<p>Welcome to your new TinyBoards instance! This pinned post walks you through everything you need to configure before opening your site to users. You can edit or delete this post at any time.</p>
<hr>
<h2>1. Configure Site Settings</h2>
<p>Head to <strong>/admin/settings</strong> to set up the basics:</p>
<ul>
<li><strong>Site name and description</strong> — appears in the header and search results</li>
<li><strong>Registration mode</strong> — choose between open registration, invite-only, application-required, or closed</li>
<li><strong>Enable/disable downvotes</strong> — decide whether your community uses downvotes</li>
<li><strong>NSFW content</strong> — toggle whether NSFW content is allowed site-wide</li>
</ul>
<h2>2. Customize Appearance</h2>
<p>Visit <strong>/admin/appearance</strong> to brand your site:</p>
<ul>
<li><strong>Site icon and banner</strong> — upload your logo and header image</li>
<li><strong>Theme colors</strong> — set primary, secondary, and hover colors</li>
<li><strong>Default theme</strong> — choose which of the 6 built-in themes new users see</li>
<li><strong>Custom CSS</strong> — add your own styles at <strong>/admin/css</strong></li>
</ul>
<h2>3. Set Up Boards</h2>
<p>Boards are the topic-based communities where content lives. You can:</p>
<ul>
<li><strong>Create new boards</strong> from the Boards directory page</li>
<li><strong>Configure each board</strong> with its own icon, banner, description, and rules</li>
<li><strong>Assign moderators</strong> to help manage individual boards</li>
<li><strong>Control board creation</strong> — in site settings, decide if all users or only admins can create boards</li>
</ul>
<h2>4. Security Checklist</h2>
<p>Before going live, make sure you've handled these:</p>
<ul>
<li>Change the default admin password to something strong</li>
<li>Set a unique <code>salt_suffix</code> in your <code>tinyboards.hjson</code> config file</li>
<li>Enable TLS (HTTPS) — set <code>tls_enabled: true</code> once your reverse proxy handles SSL</li>
<li>Update <code>cors.allowed_origins</code> to match your actual domain</li>
<li>Review rate limits in your config file and adjust for your expected traffic</li>
<li>Consider enabling CAPTCHA for registration at <strong>/admin/security</strong></li>
</ul>
<h2>5. Content &amp; Moderation</h2>
<p>Set up your moderation workflow:</p>
<ul>
<li><strong>Content filtering</strong> — configure word filters and content rules at <strong>/admin/filtering</strong></li>
<li><strong>Reports queue</strong> — review user-submitted reports at <strong>/admin/reports</strong></li>
<li><strong>Mod queue</strong> — review flagged content at <strong>/admin/queue</strong></li>
<li><strong>Site bans</strong> — manage site-wide bans at <strong>/admin/bans</strong></li>
</ul>
<h2>6. Optional: Email (SMTP)</h2>
<p>Email is needed for password resets and email verification. Add an <code>email</code> block to your <code>tinyboards.hjson</code>:</p>
<pre><code>email: {
  smtp_server: "smtp.example.com:587"
  smtp_login: "your-login"
  smtp_password: "your-password"
  smtp_from_address: "noreply@yourdomain.com"
  tls_type: "starttls"
}
</code></pre>
<p>Until email is configured, password reset and email verification will not function.</p>
<h2>7. Optional: Cloud Storage</h2>
<p>By default, uploads are stored on the local filesystem. To use S3-compatible storage (AWS S3, MinIO, DigitalOcean Spaces, Cloudflare R2), Azure Blob Storage, or Google Cloud Storage, update the <code>storage</code> block in your <code>tinyboards.hjson</code>.</p>
<h2>8. Inviting Users</h2>
<p>Depending on your registration mode:</p>
<ul>
<li><strong>Open</strong> — share your site URL and anyone can register</li>
<li><strong>Invite-only</strong> — generate invite codes at <strong>/admin/invites</strong> and distribute them</li>
<li><strong>Application-required</strong> — users submit applications; review them at <strong>/admin/applications</strong></li>
</ul>
<h2>9. Ongoing Administration</h2>
<p>Useful admin pages to bookmark:</p>
<ul>
<li><strong>/admin</strong> — dashboard overview with site statistics</li>
<li><strong>/admin/users</strong> — manage users, roles, and permissions</li>
<li><strong>/admin/admins</strong> — add or remove site administrators</li>
<li><strong>/admin/emojis</strong> — add custom emoji for your community</li>
</ul>
<hr>
<p>Once you've completed setup, feel free to unpin or delete this post. Happy community building!</p>"#;

    use diesel::sql_types::Text;
    let result = sql_query(format!(
        "INSERT INTO posts (id, title, body, body_html, creator_id, board_id, is_featured_board) \
         SELECT gen_random_uuid(), $1, $2, $3, '{admin_id}', b.id, true \
         FROM boards b WHERE b.name = '{board_name}' \
         AND NOT EXISTS (SELECT 1 FROM posts p WHERE p.title = $1 AND p.board_id = b.id)",
        admin_id = admin_id,
        board_name = board_name,
    ));

    match result
        .bind::<Text, _>(title)
        .bind::<Text, _>(body)
        .bind::<Text, _>(body_html)
        .execute(conn)
        .await
    {
        Ok(_) => info!("Welcome post created"),
        Err(e) => info!("Welcome post may already exist: {:?}", e),
    }

    // Auto-upvote the welcome post by the admin
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

    info!("Welcome post seeded successfully");
}
