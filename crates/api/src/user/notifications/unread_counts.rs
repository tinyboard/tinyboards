use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
  data::TinyBoardsContext,
  user::{GetUnreadCount, GetUnreadCountResponse},
  utils::{get_user_view_from_jwt, blocking},
};
use tinyboards_db_views::structs::{CommentReplyView, UserMentionView};
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
    let user =
        get_user_view_from_jwt(auth, context.pool(), context.master_key())
        .await?
        .user;

    let user_id = user.id;


    let replies
        = blocking(context.pool(), move |conn| {
            CommentReplyView::get_unread_replies(conn, user_id)
        })
        .await??;
    
    let mentions 
        = blocking(context.pool(), move |conn| {
            UserMentionView::get_unread_mentions(conn, user_id)
        })
        .await??;

    Ok(GetUnreadCountResponse {
        replies,
        mentions,
        total: replies + mentions,
    })
  }
}