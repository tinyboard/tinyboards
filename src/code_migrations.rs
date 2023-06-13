use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm},
        site::{site::{Site, SiteForm}, local_site::{LocalSite, LocalSiteForm}},
        person::local_user::*,
        person::person::*, apub::instance::Instance,
    },
    traits::Crud,
    utils::{naive_now, DbPool},
};
use tinyboards_api_common::utils::{
    generate_inbox_url,
    generate_local_apub_endpoint,
    generate_shared_inbox_url,
    EndpointType, generate_site_inbox_url,
};
use tinyboards_federation::http_signatures::generate_actor_keypair;
use tinyboards_utils::{
    error::TinyBoardsError, passhash::hash_password, settings::structs::Settings,
};
use tracing::info;
use url::Url;

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
    if LocalSite::read(pool).await.is_ok() {
        return Ok(());
    }
    info!("No Local Site found, initializing Tinyboards!");
    

    let domain = settings
        .get_hostname_without_port()
        .expect("must have domain");

    // upsert this to the instance table
    let instance = Instance::read_or_create(pool, domain).await?;

    if let Some(setup) = &settings.setup {
        
        let person_keypair = generate_actor_keypair()?;
        let person_actor_id = generate_local_apub_endpoint(
                EndpointType::Person, 
                &setup.admin_username, 
              &settings.get_protocol_and_hostname()
            )?;
        
        
        let person_admin_form = PersonForm::builder()
            .name(Some(setup.admin_username.clone()))
            .is_admin(Some(true))
            .actor_id(Some(person_actor_id.clone()))
            .private_key(Some(person_keypair.private_key))
            .public_key(Some(person_keypair.public_key))
            .inbox_url(Some(generate_inbox_url(&person_actor_id)?))
            .shared_inbox_url(Some(generate_shared_inbox_url(&person_actor_id)?))
            .instance_id(Some(instance.id.clone()))
            .build();

        // create the admin person object
        let inserted_admin_person = Person::create(pool, &person_admin_form).await?;

    //     let local_user_admin_form = LocalUserForm {
    //         name: Some(setup.admin_username.clone()),
    //         passhash: Some(hash_password(setup.admin_password.clone())),
    //         is_admin: Some(true),
    //         person_id: Some(inserted_admin_person.id),
    //         email: Some(setup.admin_email.clone()),
    //         ..LocalUserForm::default()
    //     };

    //     // create the local user admin object
    //     LocalUser::create(pool, &local_user_admin_form).await?;

    //     let default_name = "campfire".to_string();
    //     let default_title = "The Default Board".to_string();

    //     let default_board_form = BoardForm {
    //         name: Some(default_name),
    //         title: Some(default_title),
    //         creator_id: Some(inserted_admin_person.id.clone()),
    //         ..BoardForm::default()
    //     };

    //     // make the default board
    //     Board::create(pool, &default_board_form).await?;

    //     // add an entry to the site table
    //     let site_key_pair = generate_actor_keypair()?;
    //     let site_actor_id = Url::parse(&settings.get_protocol_and_hostname())?;

    //     let site_form = SiteForm {
    //         name: Some(settings
    //                 .setup
    //                 .clone()
    //                 .map(|s| s.site_name)
    //                 .unwrap_or_else(|| "New Site".to_string())),
    //         instance_id: Some(instance.id.clone()),
    //         actor_id: Some(site_actor_id.clone().into()),
    //         last_refreshed_date: Some(naive_now()),
    //         inbox_url: Some(generate_site_inbox_url(&site_actor_id.into())?),
    //         private_key: Some(Some(site_key_pair.private_key)),
    //         public_key: Some(site_key_pair.public_key),
    //         ..SiteForm::default()
    //     };

    //     let inserted_site = Site::create(pool, &site_form).await?;

    //     let local_site_form = LocalSiteForm {
    //         site_id: Some(inserted_site.id),
    //         site_setup: Some(settings.setup.is_some()),
    //         ..LocalSiteForm::default()
    //     };

    //     let _inserted_local_site = LocalSite::create(pool, &local_site_form).await?;
    }

    // info!("admin and site successfully initialized!");

    Ok(())
}
