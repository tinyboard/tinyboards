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
use tinyboards_db::models::user::users::User;
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

        if let Some(chat_id) = chat_id {
            
            let creator 
                = blocking(context.pool(), move |conn| {
                    User::get_user_by_chat_id(conn, chat_id)
                })
                .await??;
            
            let query_response
                = blocking(context.pool(), move |conn| {
                    PrivateMessageQuery::builder()
                        .conn(conn)
                        .recipient_id(user.id)
                        .unread_only(unread_only)
                        .creator_id(Some(creator.id))
                        .page(page)
                        .limit(limit)
                        .build()
                        .list()
                })
                .await??;
            let messages = query_response.messages;
            let total_count = query_response.count;
            let unread_count = query_response.unread;

            // mark all messages from creator as read (in the thread)
            blocking(context.pool(), move |conn| {
                PrivateMessageView::mark_thread_as_read(conn, creator.id)
            })
            .await??;
            Ok(PrivateMessagesResponse { messages, total_count, unread_count })
            
        }  else {
            let query_response
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
            let messages = query_response.messages;
            let total_count = query_response.count;
            let unread_count = query_response.unread;
            Ok(PrivateMessagesResponse { messages, total_count, unread_count })
        }
    }
}