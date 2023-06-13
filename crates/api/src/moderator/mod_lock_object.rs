use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::moderator::LockObject;
use tinyboards_api_common::moderator::UnlockObject;
use tinyboards_api_common::site::Message;
use tinyboards_api_common::utils::require_user;
use tinyboards_utils::TinyBoardsError;

use super::get_moderateable_object;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for LockObject {
    type Response = Message;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let target_object = get_moderateable_object(context.pool(), &self.target_fullname).await?;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .require_board_mod(target_object.get_board_id(), context.pool())
            .await
            .unwrap()?;

            target_object.lock(Some(view.person.id), context.pool()).await?;

        Ok(Message::new("Locked!"))
    }
}

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for UnlockObject {
    type Response = Message;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let target_object = get_moderateable_object(context.pool(), &self.target_fullname).await?;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .not_banned()
            .require_board_mod(target_object.get_board_id(), context.pool())
            .await
            .unwrap()?;

            target_object.unlock(Some(view.person.id), context.pool()).await?;

        Ok(Message::new("Unlocked!"))
    }
}
