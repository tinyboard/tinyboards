use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
  data::TinyBoardsContext,
  user::{GetUnreadCount, GetUnreadCountResponse},
  utils::{get_local_user_view_from_jwt},
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
        get_local_user_view_from_jwt(auth, context.pool(), context.master_key())
        .await?
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