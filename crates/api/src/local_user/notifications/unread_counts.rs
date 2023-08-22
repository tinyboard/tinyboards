use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
  data::TinyBoardsContext,
  person::{GetUnreadCount, GetUnreadCountResponse},
  utils::{require_user},
};
use tinyboards_db_views::structs::{CommentReplyView, PersonMentionView};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetUnreadCount {
  type Response = GetUnreadCountResponse;
  type Route = ();

  #[tracing::instrument(skip(context, auth))]
  async fn perform(
    self,
    context: &Data<TinyBoardsContext>,
    _ : Self::Route,
    auth: Option<&str>
  ) -> Result<GetUnreadCountResponse, TinyBoardsError> {
    let person =
        require_user(context.pool(), context.master_key(), auth)
        .await
        .unwrap()?
        .person;

    let person_id = person.id;


    let replies = CommentReplyView::get_unread_replies(context.pool(), person_id).await?;
    
    let mentions  = PersonMentionView::get_unread_mentions(context.pool(), person_id).await?;

    Ok(GetUnreadCountResponse {
        replies,
        mentions,
        total_count: replies + mentions,
    })
  }
}