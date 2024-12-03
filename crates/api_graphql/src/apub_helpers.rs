use anyhow::Context;
use tinyboards_db::newtypes::DbUrl;
use tinyboards_utils::error::TinyBoardsError;
use tinyboards_utils::location_info;
use url::{ParseError, Url};

pub enum EndpointType {
    Board,
    Person,
    Post,
    Comment,
}

pub fn generate_local_apub_endpoint(
    endpoint_type: EndpointType,
    name: &str,
    domain: &str,
) -> Result<DbUrl, ParseError> {
    match endpoint_type {
        EndpointType::Board => Ok(Url::parse(&format!("{domain}/+{name}"))?.into()),
        EndpointType::Comment => Ok(Url::parse(&format!("{domain}/comment/{name}"))?.into()),
        EndpointType::Post => Ok(Url::parse(&format!("{domain}/post/{name}"))?.into()),
        EndpointType::Person => Ok(Url::parse(&format!("{domain}/@{name}"))?.into()),
    }
}

pub fn generate_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/inbox"))?.into())
}

pub fn generate_outbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/outbox"))?.into())
}

pub fn generate_subscribers_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/subscribers"))?.into())
}

pub fn generate_moderators_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/mods"))?.into())
}

pub fn generate_featured_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/featured"))?.into())
}

pub fn generate_site_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    let actor_id: Url = actor_id.clone().into();
    actor_id.clone().set_path("site_inbox");
    Ok(actor_id.into())
}

pub fn generate_shared_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, TinyBoardsError> {
    let actor_id: Url = actor_id.clone().into();
    let url = format!(
        "{}://{}{}/inbox",
        &actor_id.clone().scheme(),
        &actor_id.clone().host_str().context(location_info!())?,
        if let Some(port) = actor_id.clone().port() {
            format!(":{}", port)
        } else {
            String::new()
        },
    );
    Ok(Url::parse(&url)?.into())
}
