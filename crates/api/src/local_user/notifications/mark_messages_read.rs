use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext, person::MarkAllMessagesRead, utils::require_user,
};
use tinyboards_db_views::structs::MessageView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for MarkAllMessagesRead {
    type Response = ();
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        MessageView::mark_all_messages_as_read(context.pool(), view.person.id).await?;

        Ok(())
    }
}
