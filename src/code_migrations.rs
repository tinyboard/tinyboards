// use tinyboards_api_common::utils::{
//     generate_local_apub_endpoint, EndpointType,
// };
use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm},
        user::user::{User, UserForm},
        site::site::{Site, SiteForm},
    },
    traits::Crud,
    utils::{DbPool, naive_now},
};
//use tinyboards_federation::http_signatures::generate_actor_keypair;
use tinyboards_utils::{
    error::TinyBoardsError, settings::structs::Settings, passhash::hash_password,
};
use tracing::info;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
// use tracing::info;
// use url::Url;

pub async fn run_advanced_migrations(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    initialize_core_database_records(pool).await?;
    initialize_site_and_admin_user(pool, settings).await?;

    Ok(())
}

/// Initialize core database records that are essential for the application to function
/// This includes language, JWT secret, and site aggregates
async fn initialize_core_database_records(pool: &DbPool) -> Result<(), TinyBoardsError> {
    info!("Running initialize_core_database_records");

    let mut conn = pool.get().await?;

    // Insert default English language if it doesn't exist
    sql_query("INSERT INTO language (id, code, name) VALUES (1, 'en', 'English') ON CONFLICT DO NOTHING")
        .execute(&mut conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert default language"))?;

    // Insert JWT secret if it doesn't exist (using PostgreSQL's gen_random_bytes)
    sql_query("INSERT INTO secret (id, jwt_secret) VALUES (1, encode(gen_random_bytes(32), 'hex')) ON CONFLICT DO NOTHING")
        .execute(&mut conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert JWT secret"))?;

    // Insert site aggregates if it doesn't exist (will be created after site is created)
    // This is handled after site creation in initialize_site_and_admin_user

    info!("Core database records initialized");
    Ok(())
}

/// This ensures the site is initialized
///
/// If the site is already initialized, this will not run
async fn initialize_site_and_admin_user(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    info!("Running initialize_site_and_admin_user");

    // check to see if site exists
    if Site::read(pool).await.is_ok() {
        info!("Site already initialized, skipping setup");
        return Ok(());
    }
    info!("No Site found, initializing TinyBoards!");

    if let Some(setup) = &settings.setup {
        // Create admin user first
        let user_admin_form = UserForm::builder()
            .name(Some(setup.admin_username.clone()))
            .passhash(Some(hash_password(setup.admin_password.clone())))
            .email(Some(setup.admin_email.clone()))
            .is_admin(Some(true))
            .admin_level(Some(256)) // System level admin
            .email_verified(Some(true))
            .build();

        // create the admin user object, or skip if exists
        let _inserted_admin_user = match User::create(pool, &user_admin_form).await {
            Ok(user) => {
                info!("Admin user '{}' created successfully", user.name);
                user
            },
            Err(e) => {
                info!("Admin user might already exist: {:?}", e);
                // Try to get existing user
                User::get_by_name(pool, setup.admin_username.clone()).await
                    .map_err(|_| TinyBoardsError::from_error_message(e, 500, "Failed to create or find admin user"))?
            }
        };

        // Create default board
        let board_form = BoardForm {
            name: Some(setup.default_board_name.clone()),
            title: Some(setup.default_board_name.clone()),
            description: Some(setup.default_board_description.clone()),
            instance_id: Some(1), // Default instance ID
            last_refreshed_date: Some(naive_now()),
            primary_color: Some("#3b82f6".to_string()),
            secondary_color: Some("#1e40af".to_string()),
            hover_color: Some("#2563eb".to_string()),
            ..BoardForm::default()
        };

        // make the default board, or skip if exists
        match Board::create(pool, &board_form).await {
            Ok(_) => {
                info!("Default board '{}' created successfully", setup.default_board_name);
            },
            Err(e) => {
                info!("Default board might already exist: {:?}", e);
            }
        };

        // add an entry to the site table
        let site_name = settings
            .setup
            .clone()
            .map(|s| s.site_name)
            .unwrap_or_else(|| "New TinyBoards Site".to_string());

        let site_form = SiteForm {
            name: Some(site_name.clone()),
            site_setup: Some(true),
            open_registration: Some(true),
            invite_only: Some(false),
            require_application: Some(false),
            enable_downvotes: Some(true),
            enable_nsfw: Some(true),
            board_creation_admin_only: Some(false),
            require_email_verification: Some(false),
            private_instance: Some(false),
            default_theme: Some("default".to_string()),
            default_post_listing_type: Some("Local".to_string()),
            hide_modlog_mod_names: Some(false),
            application_email_admins: Some(false),
            captcha_enabled: Some(false),
            captcha_difficulty: Some("medium".to_string()),
            reports_email_admins: Some(false),
            boards_enabled: Some(true),
            board_creation_mode: Some("AdminOnly".to_string()),
            trusted_user_min_reputation: Some(0),
            trusted_user_min_account_age_days: Some(0),
            trusted_user_manual_approval: Some(false),
            trusted_user_min_posts: Some(0),
            registration_mode: Some("Open".to_string()),
            emoji_enabled: Some(true),
            emoji_max_file_size_mb: Some(5),
            board_emojis_enabled: Some(true),
            ..SiteForm::default()
        };

        let _inserted_site = Site::create(pool, &site_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create site"))?;

        // Initialize site aggregates after site creation
        let mut conn = pool.get().await?;
        sql_query("INSERT INTO site_aggregates (site_id) VALUES (1) ON CONFLICT DO NOTHING")
            .execute(&mut conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to insert site aggregates"))?;

        info!("Site '{}' and admin user successfully initialized!", site_name);

        Ok(())
    } else {
        info!("No setup configuration found in settings, skipping initialization");
        Ok(())
    }
}
