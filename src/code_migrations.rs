use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm},
        site::site::{Site, SiteForm},
        person::local_user::*,
        person::person::*,
    },
    traits::Crud,
    utils::{naive_now, DbPool},
};
use tinyboards_api_common::utils::{
    generate_inbox_url,
    generate_local_apud_endpoint,
    generate_shared_inbox_url,
    EndpointType,
};
use tinyboards_federation::http_signatures::generate_actor_keypair;
use tinyboards_utils::{
    error::TinyBoardsError, passhash::hash_password, settings::structs::Settings,
};
use tracing::info;

pub async fn run_advanced_migrations(
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    initialize_local_site_and_admin_user(pool, settings).await?;

    Ok(())
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
        
        let local_admin_form = LocalUserForm {
            name: Some(setup.admin_username.clone()),
            passhash: Some(hash_password(setup.admin_password.clone())),
            is_admin: Some(true),
            ..LocalUserForm::default()
        };

        // create the local user admin object
        LocalUser::create(pool, &local_admin_form).await?;

        // generate the actor keypair for the admin user
        let admin_keypair = generate_actor_keypair()?;

        // generate the actor_id for the admin user
        let actor_id = generate_local_apud_endpoint(
            EndpointType::Person, 
            &setup.admin_username,
            &settings.get_protocol_and_hostname(),
        )?;

        // make the admin person form
        let person_admin_form = PersonForm {
            name: Some(setup.admin_username.clone()),
            actor_id: Some(actor_id.to_string().clone()),
            private_key: Some(Some(admin_keypair.private_key)),
            public_key: Some(Some(admin_keypair.public_key)),
            inbox_url: Some(generate_inbox_url(&actor_id)?.to_string()),
            shared_inbox_url: Some(Some(generate_shared_inbox_url(&actor_id)?.to_string())),
            ..PersonForm::default()
        };

        // create the admin person object
        let inserted_admin_person = Person::create(pool, &person_admin_form).await?;

        let default_name = "campfire".to_string();
        let default_title = "The Default Board".to_string();

        let default_board_form = BoardForm {
            name: Some(default_name),
            title: Some(default_title),
            creator_id: Some(inserted_admin_person.id.clone()),
            ..BoardForm::default()
        };

        // make the default board
        Board::create(pool, &default_board_form).await?;

        let site_form = SiteForm {
            name: Some(setup.site_name.clone()),
            description: None,
            creator_id: Some(inserted_admin_person.id),
            updated: Some(Some(naive_now())),
            ..SiteForm::default()
        };

        // initialize the site
        Site::create(pool, &site_form).await?;
    }

    info!("admin and site successfully initialized!");

    Ok(())
}
