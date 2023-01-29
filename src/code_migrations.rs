use tinyboards_api_common::utils::blocking;
use tinyboards_db::{
    database::PgPool,
    models::{
        board::boards::{Board, BoardForm},
        site::site::{Site, SiteForm},
        user::users::{User, UserForm},
    },
    traits::Crud,
    utils::naive_now,
};
use tinyboards_utils::{
    error::TinyBoardsError, passhash::hash_password, settings::structs::Settings, utils::generate_rand_string,
};
use tracing::info;

pub async fn run_advanced_migrations(
    pool: &PgPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    initialize_local_site_and_admin_user(pool, settings).await?;
    generate_chat_ids_for_users(pool).await?;

    Ok(())
}

/// This ensures every user has a chat_id, it's fine to run every time to ensure everyone has a proper id
async fn generate_chat_ids_for_users(
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    info!("Running generate_chat_ids_for_users");

    let users = blocking(pool, move |conn| {
        User::get_users_by_chat_id(conn, String::from("n/a"))
    })
    .await??;

    if users.len() == 0 {
        info!("No chat ids needed to be generated, proceeding...");
        Ok(())
    } else {
        for user in users.into_iter() {
            let chat_id = generate_rand_string();
            blocking(pool, move |conn| {
                User::update_chat_id(conn, user.id, chat_id)
            })
            .await??;
        }
        info!("Successfully generated chat ids, proceeding...");
        Ok(())
    }
}

/// This ensures the site is initialized
///
/// If the site is already initialized, this will not run
async fn initialize_local_site_and_admin_user(
    pool: &PgPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    info!("Running initialize_local_site_and_admin_user");

    // check to see if local site exists
    let exists = blocking(pool, move |conn| Site::exists(conn))
        .await?
        .is_ok();

    if exists {
        return Ok(());
    }
    info!("Site not initialized yet, creating it.");

    let _domain = settings
        .get_hostname_without_port()
        .expect("must have domain");

    if let Some(setup) = &settings.setup {
        let user_form = UserForm {
            name: Some(setup.admin_username.clone()),
            passhash: Some(hash_password(setup.admin_password.clone())),
            is_admin: Some(true),
            ..UserForm::default()
        };

        let inserted_admin = blocking(pool, move |conn| User::create(conn, &user_form)).await??;

        let default_name = "campfire".to_string();
        let default_title = "The Default Board".to_string();

        let default_board_form = BoardForm {
            name: Some(default_name),
            title: Some(default_title),
            creator_id: Some(inserted_admin.id.clone()),
            ..BoardForm::default()
        };

        // make the default board
        blocking(pool, move |conn| Board::create(conn, &default_board_form)).await??;

        let site_form = SiteForm {
            name: Some(setup.site_name.clone()),
            creator_id: Some(inserted_admin.id),
            updated: Some(Some(naive_now())),
            enable_downvotes: Some(true),
            enable_nsfw: Some(true),
            email_verification_required: Some(false),
            open_registration: Some(true),
            require_application: Some(false),
            invite_only: Some(false),
            private_instance: Some(false),
            ..SiteForm::default()
        };

        // initialize the site
        blocking(pool, move |conn| Site::create(conn, &site_form)).await??;
    }

    info!("admin and site successfully initialized!");

    Ok(())
}
