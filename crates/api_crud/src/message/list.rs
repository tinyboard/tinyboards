use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    message::{GetMessages, GetMessagesResponse},
    utils::require_user,
};
use tinyboards_db_views::{message_view::{MessageQuery, MessageQueryResponse}, structs::MessageView};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetMessages {
    type Response = GetMessagesResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let v = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let (page, limit) = (self.page, self.limit);
        let unread_only = self.unread_only;
        let board_id = self.board_id;

        let MessageQueryResponse {
            messages,
            count,
            unread,
        } = MessageQuery::builder()
            .pool(context.pool())
            .person_id(v.person.id)
            .board_id(board_id)
            .unread_only(unread_only)
            .page(page)
            .limit(limit)
            .build()
            .list()
            .await?;

        // mark all messages as read when visiting it in the inbox
        MessageView::mark_all_messages_as_read(context.pool(), v.person.id.clone()).await?;

        Ok(GetMessagesResponse {
            messages,
            count,
            unread,
        })
    }
}
