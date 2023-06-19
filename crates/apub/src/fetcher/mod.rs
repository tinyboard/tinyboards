use tinyboards_federation::{
    config::Data,
    fetch::webfinger::webfinger_resolve_actor,
    traits::{Actor, Object},
  };
  use diesel::NotFound;
  use itertools::Itertools;
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::traits::ApubActor;
  use tinyboards_db_views::structs::LocalUserView;
  use tinyboards_utils::error::TinyBoardsError;
  
  pub mod post_or_comment;
  pub mod search;
  pub mod user_or_board;
  
  /// Resolve actor identifier like `!news@example.com` to user or community object.
  ///
  /// In case the requesting user is logged in and the object was not found locally, it is attempted
  /// to fetch via webfinger from the original instance.
  #[tracing::instrument(skip_all)]
  pub async fn resolve_actor_identifier<ActorType, DbActor>(
    identifier: &str,
    context: &Data<TinyBoardsContext>,
    local_user_view: &Option<LocalUserView>,
    include_deleted: bool,
  ) -> Result<ActorType, TinyBoardsError>
  where
    ActorType: Object<DataType = TinyBoardsContext, Error = TinyBoardsError>
      + Object
      + Actor
      + From<DbActor>
      + Send
      + 'static,
    for<'de2> <ActorType as Object>::Kind: serde::Deserialize<'de2>,
    DbActor: ApubActor + Send + 'static,
  {
    // remote actor
    if identifier.contains('@') {
      let (name, domain) = identifier
        .splitn(2, '@')
        .collect_tuple()
        .expect("invalid query");
      let actor = DbActor::read_from_name_and_domain(context.pool(), name, domain).await;
      if actor.is_ok() {
        Ok(actor?.into())
      } else if local_user_view.is_some() {
        // Fetch the actor from its home instance using webfinger
        let actor: ActorType = webfinger_resolve_actor(identifier, context).await?;
        Ok(actor)
      } else {
        Err(NotFound.into())
      }
    }
    // local actor
    else {
      let identifier = identifier.to_string();
      Ok(
        DbActor::read_from_name(context.pool(), &identifier, include_deleted)
          .await?
          .into(),
      )
    }
  }