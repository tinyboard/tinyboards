use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{BanFromBoard, BanFromBoardResponse},
    utils::require_user,
};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::board::{
        board_person_bans::{BoardPersonBan, BoardPersonBanForm},
        board_subscriber::{BoardSubscriber, BoardSubscriberForm},
    },
    models::moderator::mod_actions::{ModBanFromBoard, ModBanFromBoardForm},
    traits::{Bannable, Crud, Subscribeable},
};
use tinyboards_db_views::structs::PersonView;
use tinyboards_utils::{error::TinyBoardsError, time::naive_from_unix};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for BanFromBoard {
    type Response = BanFromBoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &BanFromBoard = &self;

        let target_person_id = data.person_id;
        let board_id = data.board_id;
        let reason = data.reason.clone();
        let expires = Some(naive_from_unix(data.expires.unwrap()));
        let banned = data.ban;

        // require board moderator (at least) to perform this action
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), board_id.clone(), ModPerms::Users, None)
            .await
            .unwrap()?;

        let board_user_ban_form = BoardPersonBanForm {
            board_id: board_id.clone(),
            person_id: target_person_id.clone(),
            expires,
        };

        if banned {
            // ban user from board
            BoardPersonBan::ban(context.pool(), &board_user_ban_form).await?;

            // also unsubscribe them from board, if subbed
            let sub_form = BoardSubscriberForm {
                board_id: board_id.clone(),
                person_id: target_person_id.clone(),
                pending: None,
            };
            BoardSubscriber::unsubscribe(context.pool(), &sub_form).await?;
        } else {
            // unban user from board
            BoardPersonBan::unban(context.pool(), &board_user_ban_form).await?;
        }

        // mod log form
        let ban_from_board_form = ModBanFromBoardForm {
            mod_person_id: view.person.id,
            other_person_id: target_person_id,
            board_id,
            reason: Some(reason),
            banned: Some(Some(banned)),
            expires: Some(expires),
        };

        // mod log entry
        ModBanFromBoard::create(context.pool(), &ban_from_board_form).await?;

        // get person_view for response
        let person_id = data.person_id;
        let person_view = PersonView::read(context.pool(), person_id, false).await?;

        Ok(BanFromBoardResponse {
            person_view,
            banned: data.ban,
        })
    }
}
