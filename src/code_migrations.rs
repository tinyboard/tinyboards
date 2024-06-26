use tinyboards_api_common::utils::{
    generate_inbox_url, generate_local_apub_endpoint, generate_shared_inbox_url,
    generate_site_inbox_url, generate_subscribers_url, EndpointType,
};
use tinyboards_db::{
    models::{
        apub::instance::Instance,
        board::boards::{Board, BoardForm},
        person::local_user::*,
        person::person::*,
        site::{
            local_site::{LocalSite, LocalSiteForm},
            local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitForm},
            site::{Site, SiteForm},
        },
    },
    traits::Crud,
    utils::{naive_now, DbPool},
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
            &settings.get_protocol_and_hostname(),
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

        let local_user_admin_form = LocalUserForm {
            name: Some(setup.admin_username.clone()),
            passhash: Some(hash_password(setup.admin_password.clone())),
            // for the time being, the first account is the system acc
            admin_level: Some(256),
            person_id: Some(inserted_admin_person.id),
            email: Some(setup.admin_email.clone()),
            ..LocalUserForm::default()
        };

        // create the local user admin object
        LocalUser::create(pool, &local_user_admin_form).await?;

        let board_key_pair = generate_actor_keypair()?;
        let board_actor_id = generate_local_apub_endpoint(
            EndpointType::Board,
            &setup.default_board_name.clone(),
            &settings.get_protocol_and_hostname(),
        )?;

        let board_form = BoardForm {
            name: Some(setup.default_board_name.clone()),
            title: Some(setup.default_board_name.clone()),
            description: Some(setup.default_board_description.clone()),
            public_key: Some(board_key_pair.public_key),
            private_key: Some(board_key_pair.private_key),
            actor_id: Some(board_actor_id.clone()),
            subscribers_url: Some(generate_subscribers_url(&board_actor_id.clone())?),
            inbox_url: Some(generate_inbox_url(&board_actor_id.clone())?),
            shared_inbox_url: Some(Some(generate_shared_inbox_url(&board_actor_id.clone())?)),
            instance_id: Some(instance.id.clone()),
            ..BoardForm::default()
        };

        // make the default board
        Board::create(pool, &board_form).await?;

        // add an entry to the site table
        let site_key_pair = generate_actor_keypair()?;
        let site_actor_id = Url::parse(&settings.get_protocol_and_hostname())?;
        let site_name = settings
            .setup
            .clone()
            .map(|s| s.site_name)
            .unwrap_or_else(|| "New Site".to_string());

        let site_form = SiteForm {
            name: Some(site_name.clone()),
            instance_id: Some(instance.id.clone()),
            actor_id: Some(site_actor_id.clone().into()),
            last_refreshed_date: Some(naive_now()),
            inbox_url: Some(generate_site_inbox_url(&site_actor_id.into())?),
            private_key: Some(Some(site_key_pair.private_key)),
            public_key: Some(site_key_pair.public_key),
            ..SiteForm::default()
        };

        let inserted_site = Site::create(pool, &site_form).await?;

        let local_site_form = LocalSiteForm {
            site_id: Some(inserted_site.id),
            name: Some(site_name),
            site_setup: Some(settings.setup.is_some()),
            open_registration: Some(true),
            invite_only: Some(false),
            require_application: Some(false),
            ..LocalSiteForm::default()
        };

        let inserted_local_site = LocalSite::create(pool, &local_site_form).await?;

        let local_site_rate_limit_form = LocalSiteRateLimitForm {
            message: Some(999),
            post: Some(999),
            register: Some(999),
            image: Some(999),
            comment: Some(999),
            search: Some(999),
            local_site_id: Some(inserted_local_site.id),
            ..LocalSiteRateLimitForm::default()
        };

        LocalSiteRateLimit::create(pool, &local_site_rate_limit_form).await?;

        info!("admin and site successfully initialized!");

        Ok(())
    } else {
        Ok(())
    }
}
