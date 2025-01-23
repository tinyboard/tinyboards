use crate::{
    activity_lists::PersonInboxActivitiesWithAnnouncable,
    fetcher::user_or_board::UserOrBoard,
    http::{create_apub_response, create_apub_tombstone_response},
    objects::person::ApubPerson,
    protocol::collections::empty_outbox::EmptyOutbox,
};
use tinyboards_federation::{
    actix_web::inbox::receive_activity,
    config::Data,
    protocol::context::WithContext,
    traits::Object,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use tinyboards_api_common::{data::TinyBoardsContext, utils::generate_outbox_url};
use tinyboards_db::{models::person::person::Person, traits::ApubActor};
use tinyboards_utils::error::TinyBoardsError;
use serde::Deserialize;
  
  #[derive(Deserialize)]
  pub struct PersonQuery {
    user_name: String,
  }
  
  /// Return the ActivityPub json representation of a local person over HTTP.
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_apub_person_http(
    info: web::Path<PersonQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let user_name = info.into_inner().user_name;
    // TODO: this needs to be able to read deleted persons, so that it can send tombstones
    let person: ApubPerson = Person::read_from_name(context.pool(), &user_name, true)
      .await?
      .into();
  
    if !person.is_deleted {
      let apub = person.into_json(&context).await?;
  
      create_apub_response(&apub)
    } else {
      create_apub_tombstone_response(person.actor_id.clone())
    }
  }
  
  #[tracing::instrument(skip_all)]
  pub async fn person_inbox(
    request: HttpRequest,
    body: Bytes,
    data: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    receive_activity::<WithContext<PersonInboxActivitiesWithAnnouncable>, UserOrBoard, TinyBoardsContext>(
      request, body, &data,
    )
    .await
  }
  
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_apub_person_outbox(
    info: web::Path<PersonQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let person = Person::read_from_name(context.pool(), &info.user_name, false).await?;
    let outbox_id = generate_outbox_url(&person.actor_id)?.into();
    let outbox = EmptyOutbox::new(outbox_id)?;
    create_apub_response(&outbox)
}