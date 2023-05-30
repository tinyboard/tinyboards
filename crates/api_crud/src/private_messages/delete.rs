use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    private_messages::{DeletePrivateMessage, PrivateMessageResponse, PrivateMessageIdPath},
    utils::{
        require_user,
    },
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::{
        local_user::private_messages::{PrivateMessage, PrivateMessageForm},
    },
    traits::Crud, utils::naive_now,
};
use tinyboards_db_views::structs::PrivateMessageView;
use tinyboards_utils::{TinyBoardsError};


#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeletePrivateMessage {
    type Response = PrivateMessageResponse;
    type Route = PrivateMessageIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<PrivateMessageResponse, TinyBoardsError> {
        let data: &DeletePrivateMessage = &self;

        let is_deleted = data.is_deleted.clone();

        let pm_id = path.pm_id.clone();

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let orig_pm = PrivateMessage::read(context.pool(), pm_id).await?;

        if user.id != orig_pm.creator_id {
            return Err(TinyBoardsError::from_message(403, "private message edit not allowed"));
        }

        let form = PrivateMessageForm {
            creator_id: Some(orig_pm.creator_id),
            recipient_id: Some(orig_pm.recipient_id),
            body: Some(orig_pm.body),
            is_parent: Some(orig_pm.is_parent),
            is_deleted: Some(is_deleted),
            read: Some(orig_pm.read),
            updated: Some(naive_now()),
        };

        PrivateMessage::update(context.pool(), pm_id, &form).await?;

        let message = PrivateMessageView::read(context.pool(), pm_id).await?;

        Ok(PrivateMessageResponse { message })

    }
}