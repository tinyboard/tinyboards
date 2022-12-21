use tinyboards_db::{
    models::{
        site::site::{Site, SiteForm}, user::user::{UserForm, User},
    }, 
    database::PgPool,
    traits::Crud, utils::naive_now,
};
use tinyboards_api_common::{utils::blocking};
use tinyboards_utils::{error::TinyBoardsError, settings::structs::Settings, passhash::hash_password};
use tracing::info;

pub async fn run_advanced_migrations(pool: &PgPool, settings: &Settings) -> Result<(), TinyBoardsError> {

    initialize_local_site_and_admin_user(pool, settings).await?;

    Ok(())
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
    let exists = blocking(pool, move |conn| {
        Site::exists(conn)
    })
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
            name: setup.admin_username.clone(),
            passhash: hash_password(setup.admin_password.clone()),
            admin: Some(true),
            ..UserForm::default()
        };

        let inserted_admin = blocking(pool, move |conn| {
            User::create(conn, &user_form)
        })
        .await??;
        
        let site_form = SiteForm{
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
        blocking(pool, move |conn| {
            Site::create(conn, &site_form)
        })
        .await??;
    }

    info!("admin and site successfully initialized!");

    Ok(())
}