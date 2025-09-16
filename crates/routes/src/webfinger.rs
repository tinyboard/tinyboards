/*use tinyboards_federation::{
    config::Data,
    fetch::webfinger::{extract_webfinger_name, Webfinger, WebfingerLink},
};
use actix_web::{web, web::Query, HttpResponse};
use tinyboards_api_graphql::context::TinyBoardsContext;
use tinyboards_db::{
    models::{board::boards::Board, person::person::Person},
    traits::ApubActor,
};
use tinyboards_utils::error::TinyBoardsError;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Deserialize)]
struct Params {
    resource: String,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        ".well-known/webfinger",
        web::get().to(get_webfinger_response)
    );
}

/// Responds to webfinger requests of the following format.
/// https://mastodon.social/.well-known/webfinger?resource=acct:gargron@mastodon.social
///
/// You can also view the webfinger response that Mastodon sends:
/// https://radical.town/.well-known/webfinger?resource=acct:felix@radical.town
async fn get_webfinger_response(
    info: Query<Params>,
    context: Data<TinyBoardsContext>
) -> Result<HttpResponse, TinyBoardsError> {
    let name = extract_webfinger_name(&info.resource, &context)?;
    let name_ = name.clone();

    let user_id: Option<Url> = Person::read_from_name(context.pool(), &name_, false)
        .await
        .ok()
        .map(|c| c.actor_id.into());
    let board_id: Option<Url> = Board::read_from_name(context.pool(), &name, false)
        .await
        .ok()
        .map(|c| c.actor_id.into());

    let links = vec![
        webfinger_link_for_actor(user_id, "Person"),
        webfinger_link_for_actor(board_id, "Group"),
    ]
    .into_iter()
    .flatten()
    .collect();

    let json = Webfinger {
        subject: info.resource.clone(),
        links,
        ..Default::default()
    };

    Ok(HttpResponse::Ok().json(json))
}


fn webfinger_link_for_actor(url: Option<Url>, kind: &str) -> Vec<WebfingerLink> {
    if let Some(url) = url {
      let mut properties = HashMap::new();
      properties.insert(
        "https://www.w3.org/ns/activitystreams#type"
          .parse()
          .expect("parse url"),
        kind.to_string(),
      );
      vec![
        WebfingerLink {
          rel: Some("http://webfinger.net/rel/profile-page".to_string()),
          kind: Some("text/html".to_string()),
          href: Some(url.clone()),
          ..Default::default()
        },
        WebfingerLink {
          rel: Some("self".to_string()),
          kind: Some("application/activity+json".to_string()),
          href: Some(url),
          properties,
        },
      ]
    } else {
      vec![]
    }
  }*/
