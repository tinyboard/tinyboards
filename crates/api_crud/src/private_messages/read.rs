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
use tinyboards_db_views::{private_message_view::PrivateMessageQuery, structs::PrivateMessageView};
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
        let chat_id = data.chat_id.clone();

        let query_response
            = blocking(context.pool(), move |conn| {
                PrivateMessageQuery::builder()
                    .conn(conn)
                    .recipient_id(user.id)
                    .unread_only(unread_only)
                    .chat_id(chat_id)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
            })
            .await??;

        let mut messages = query_response.messages;
        let total_count = query_response.count;
        let unread_count = query_response.unread;

        // mark all messages sent by user as read (cosmetically)
        messages.iter_mut().for_each(|pm| {
            if pm.creator.id == user.id {
                pm.private_message.read = true
            }
        });

        let chat_id = data.chat_id.clone();
        // actually mark thread as read like a boss 
        if chat_id.is_some() {
            blocking(context.pool(), move |conn| {
                PrivateMessageView::mark_thread_as_read(conn, chat_id.unwrap())
            })
            .await??;
        }

        Ok(PrivateMessagesResponse { messages, total_count, unread_count })
    }
}