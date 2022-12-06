use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{BanFromBoard, ModActionResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::moderator::mod_actions::{ModBanFromBoard, ModBanFromBoardForm},
    models::{
        board::{
            board_user_ban::{BoardUserBan, BoardUserBanForm}, 
            board_subscriber::{BoardSubscriberForm, BoardSubscriber}
        }
    },
    traits::{Crud, Bannable, Subscribeable},
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for BanFromBoard {
    type Response = ModActionResponse<ModBanFromBoard>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &BanFromBoard = &self;

        let target_user_id = data.target_user_id;
        let board_id = data.board_id;
        let reason = data.reason.clone();
        let expires = data.expires;
        let banned = data.banned;

        // require board moderator (at least) to perform this action
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(board_id.clone(), context.pool())
            .await
            .unwrap()?;

        let board_user_ban_form = BoardUserBanForm {
            board_id: board_id.clone(),
            user_id: target_user_id.clone(),
            expires: expires.clone(),
        };

        
        if banned {
            // ban user from board
            blocking(context.pool(), move |conn| {
                BoardUserBan::ban(conn, &board_user_ban_form)
            })
            .await??;

            // also unsubscribe them from board, if subbed
            let sub_form = BoardSubscriberForm {
                board_id: board_id.clone(),
                user_id: target_user_id.clone(),
                pending: None,
            };

            blocking(context.pool(), move |conn| {
                BoardSubscriber::unsubscribe(conn, &sub_form)
            })
            .await??;

        } else {
            // unban user from board
            blocking(context.pool(), move |conn| {
                BoardUserBan::unban(conn, &board_user_ban_form)
            })
            .await??;
        }

        // mod log form
        let ban_from_board_form = ModBanFromBoardForm {
            mod_user_id: user.id,
            other_user_id: target_user_id,
            board_id,
            reason: Some(reason),
            banned: Some(Some(banned)),
            expires: Some(expires),
        };

        let mod_action = blocking(context.pool(), move |conn| {
            ModBanFromBoard::create(conn, &ban_from_board_form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}