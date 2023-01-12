use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    private_messages::{GetPrivateMessages, PrivateMessagesResponse},
    utils::{
        blocking,
        require_user,
    },
    data::TinyBoardsContext,
};
use tinyboards_db_views::{private_message_view::PrivateMessageQuery};
use tinyboards_utils::{TinyBoardsError};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPrivateMessages {
    type Response = PrivateMessagesResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<PrivateMessagesResponse, TinyBoardsError> {
        let data: &GetPrivateMessages = &self;

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let page = data.page;
        let limit = data.limit;
        let unread_only = data.unread_only;

        let mut messages
            = blocking(context.pool(), move |conn| {
                PrivateMessageQuery::builder()
                    .conn(conn)
                    .recipient_id(user.id)
                    .unread_only(unread_only)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
            })
            .await??;
        
        // mark all messages sent by user as read (cosmetically)
        messages.iter_mut().for_each(|pm| {
            if pm.creator.id == user.id {
                pm.private_message.read = true
            }
        });

        Ok(PrivateMessagesResponse { messages })
    }
}