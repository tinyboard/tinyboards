use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::moderator::ApproveObject;
use tinyboards_api_common::utils::require_user;
use tinyboards_api_common::{moderator::RemoveObject, site::Message};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_utils::TinyBoardsError;

use super::get_moderateable_object;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for RemoveObject {
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
            .require_board_mod(
                context.pool(),
                target_object.get_board_id(),
                ModPerms::Content,
            )
            .await
            .unwrap()?;

        target_object
            .remove(Some(view.person.id), self.reason, context.pool())
            .await?;

        Ok(Message::new("Removed!"))
    }
}

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for ApproveObject {
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
            .require_board_mod(
                context.pool(),
                target_object.get_board_id(),
                ModPerms::Content,
            )
            .await
            .unwrap()?;

        target_object
            .approve(Some(view.person.id), context.pool())
            .await?;

        Ok(Message::new("Approved!"))
    }
}
