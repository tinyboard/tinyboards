use tinyboards_federation::kinds::collection::CollectionType;
use tinyboards_api_common::{data::TinyBoardsContext, utils::generate_subscribers_url};
use tinyboards_db::{models::board::boards::Board};
use tinyboards_db_views::structs::BoardSubscriberView;
use tinyboards_utils::error::TinyBoardsError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroupSubscribers {
  id: Url,
  r#type: CollectionType,
  total_items: i32,
  items: Vec<()>,
}

impl GroupSubscribers {
  pub(crate) async fn new(
    board: Board,
    context: &TinyBoardsContext,
  ) -> Result<GroupSubscribers, TinyBoardsError> {
    let board_id = board.id;
    let board_subscribers =
      BoardSubscriberView::for_board(context.pool(), board_id).await?;

    Ok(GroupSubscribers {
      id: generate_subscribers_url(&board.actor_id)?.into(),
      r#type: CollectionType::Collection,
      total_items: board_subscribers.len() as i32,
      items: vec![],
    })
  }
}