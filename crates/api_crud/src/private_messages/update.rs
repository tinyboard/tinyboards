use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    private_messages::{EditPrivateMessage, PrivateMessageResponse, PrivateMessageIdPath},
    utils::{
        blocking,
        require_user,
    },
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::{
        user::private_messages::{PrivateMessage, PrivateMessageForm},
    },
    traits::Crud, utils::naive_now,
};
use tinyboards_db_views::structs::PrivateMessageView;
use tinyboards_utils::{parser::parse_markdown, TinyBoardsError};


#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditPrivateMessage {
    type Response = PrivateMessageResponse;
    type Route = PrivateMessageIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<PrivateMessageResponse, TinyBoardsError> {

        let data: &EditPrivateMessage = &self;

        let pm_id = path.pm_id.clone();

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let orig_pm = blocking(context.pool(), move |conn| {
            PrivateMessage::read(conn, pm_id)
        })
        .await??;

        if user.id != orig_pm.creator_id {
            return Err(TinyBoardsError::from_message(403, "private message edit not allowed"));
        }

        let body = data.body.clone();
        let body_parsed = parse_markdown(&body.as_str());

        let updated = Some(naive_now());

        let form = PrivateMessageForm {
            creator_id: Some(orig_pm.creator_id),
            recipient_id: Some(orig_pm.recipient_id),
            body: body_parsed,
            is_parent: Some(orig_pm.is_parent),
            is_deleted: Some(orig_pm.is_deleted),
            read: Some(orig_pm.read),
            updated,
        };

        blocking(context.pool(), move |conn| {
            PrivateMessage::update(conn, pm_id, &form)
        })
        .await??;

        let message = blocking(context.pool(), move |conn| {
            PrivateMessageView::read(conn, pm_id)
        })
        .await??;

        Ok(PrivateMessageResponse { message })
    }
}