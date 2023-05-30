use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm},
        site::site::{Site, SiteForm},
        local_user::users::{User, UserForm},
    },
    traits::Crud,
    utils::{naive_now, DbPool},
};
use tinyboards_utils::{
    error::TinyBoardsError, passhash::hash_password, settings::structs::Settings, utils::generate_rand_string,
};
use tracing::info;

pub async fn run_advanced_migrations(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    initialize_local_site_and_admin_user(pool, settings).await?;
    generate_chat_ids_for_users(pool).await?;

    Ok(())
}

/// This ensures every user has a chat_id, it's fine to run every time to ensure everyone has a proper id
async fn generate_chat_ids_for_users(
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    info!("Running generate_chat_ids_for_users");

    let users = User::get_users_by_chat_id(pool, String::from("n/a")).await?;

    if users.len() == 0 {
        info!("No chat ids needed to be generated, proceeding...");
        Ok(())
    } else {
        for user in users.into_iter() {
            let chat_id = generate_rand_string();
            User::update_chat_id(pool, user.id, chat_id).await?;
        }
        info!("Successfully generated chat ids, proceeding...");
        Ok(())
    }
}

/// This ensures the site is initialized
///
/// If the site is already initialized, this will not run
async fn initialize_local_site_and_admin_user(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    info!("Running initialize_local_site_and_admin_user");

    // check to see if local site exists
    let exists = Site::exists(pool)
        .await
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

        let inserted_admin = User::create(pool, &user_form).await?;

        let default_name = "campfire".to_string();
        let default_title = "The Default Board".to_string();

        let default_board_form = BoardForm {
            name: Some(default_name),
            title: Some(default_title),
            creator_id: Some(inserted_admin.id.clone()),
            ..BoardForm::default()
        };

        // make the default board
        Board::create(pool, &default_board_form).await?;

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
        Site::create(pool, &site_form).await?;
    }

    info!("admin and site successfully initialized!");

    Ok(())
}
