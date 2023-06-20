use crate::{
    activity_lists::GroupInboxActivities,
    collections::{
      board_featured::ApubBoardFeatured,
      board_moderators::ApubBoardModerators,
      board_outbox::ApubBoardOutbox,
    },
    http::{create_apub_response, create_apub_tombstone_response},
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::collections::group_subscribers::GroupSubscribers,
  };
  use tinyboards_federation::{
    actix_web::inbox::receive_activity,
    config::Data,
    protocol::context::WithContext,
    traits::{Collection, Object},
  };
  use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{models::board::boards::Board, traits::ApubActor};
  use tinyboards_utils::error::TinyBoardsError;
  use serde::Deserialize;
  
  #[derive(Deserialize)]
  pub(crate) struct BoardQuery {
    board_name: String,
  }
  
  /// Return the ActivityPub json representation of a local board over HTTP.
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_apub_board_http(
    info: web::Path<BoardQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let board: ApubBoard =
      Board::read_from_name(context.pool(), &info.board_name, true)
        .await?
        .into();
  
    if !board.is_deleted && !board.is_removed {
      let apub = board.into_json(&context).await?;
  
      create_apub_response(&apub)
    } else {
      create_apub_tombstone_response(board.actor_id.clone())
    }
  }
  
  /// Handler for all incoming receive to board inboxes.
  #[tracing::instrument(skip_all)]
  pub async fn board_inbox(
    request: HttpRequest,
    body: Bytes,
    data: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    receive_activity::<WithContext<GroupInboxActivities>, ApubPerson, TinyBoardsContext>(
      request, body, &data,
    )
    .await
  }
  
  /// Returns an empty subscribers collection, only populating the size (for privacy).
  pub(crate) async fn get_apub_board_followers(
    info: web::Path<BoardQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let board = Board::read_from_name(context.pool(), &info.board_name, false).await?;
    let subscribers = GroupSubscribers::new(board, &context).await?;
    create_apub_response(&subscribers)
  }
  
  /// Returns the board outbox, which is populated by a maximum of 20 posts (but no other
  /// activites like votes or comments).
  pub(crate) async fn get_apub_board_outbox(
    info: web::Path<BoardQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let board: ApubBoard =
      Board::read_from_name(context.pool(), &info.board_name, false)
        .await?
        .into();
    if board.is_deleted || board.is_removed {
      return Err(TinyBoardsError::from_message(400, "board deleted"));
    }
    let outbox = ApubBoardOutbox::read_local(&board, &context).await?;
    create_apub_response(&outbox)
  }
  
  #[tracing::instrument(skip_all)]
  pub(crate) async fn get_apub_board_moderators(
    info: web::Path<BoardQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let board: ApubBoard =
      Board::read_from_name(context.pool(), &info.board_name, false)
        .await?
        .into();
    if board.is_deleted || board.is_removed {
      return Err(TinyBoardsError::from_message(400, "board deleted"));
    }
    let moderators = ApubBoardModerators::read_local(&board, &context).await?;
    create_apub_response(&moderators)
  }
  
  /// Returns collection of featured (stickied) posts.
  pub(crate) async fn get_apub_board_featured(
    info: web::Path<BoardQuery>,
    context: Data<TinyBoardsContext>,
  ) -> Result<HttpResponse, TinyBoardsError> {
    let board: ApubBoard =
      Board::read_from_name(context.pool(), &info.board_name, false)
        .await?
        .into();
    if board.is_deleted || board.is_removed {
      return Err(TinyBoardsError::from_message(400, "board deleted"));
    }
    let featured = ApubBoardFeatured::read_local(&board, &context).await?;
    create_apub_response(&featured)
  }