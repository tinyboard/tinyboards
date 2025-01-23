use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::utils::load_user_opt;
use tinyboards_api_common::{
    board::{BoardIdPath, ListBoardMods, ListBoardModsResponse},
    data::TinyBoardsContext,
};
/*use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
        moderator::mod_actions::{ModAddBoardMod, ModAddBoardModForm},
        person::local_user::AdminPerms,
    },
    traits::Crud,
};*/
use tinyboards_db_views::structs::{BoardModeratorView, LocalUserView};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListBoardMods {
    type Route = BoardIdPath;
    type Response = ListBoardModsResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        BoardIdPath { board_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        //let data = &self;

        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;

        let mods = BoardModeratorView::for_board(context.pool(), board_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load board mods.")
            })?;

        let pending_mods = BoardModeratorView::get_board_pending_invites(context.pool(), board_id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load board pending mods.")
            })?;

        let has_pending_invite = if let Some(LocalUserView { ref person, .. }) = v {
            pending_mods
                .iter()
                .map(|mod_view| mod_view.moderator.id)
                .collect::<Vec<i32>>()
                .contains(&person.id)
        } else {
            false
        };

        let my_mod_rank: Option<i32> = if let Some(LocalUserView { ref person, .. }) = v {
            mods.iter()
                .map(|mod_view| (mod_view.moderator.id, mod_view.mod_meta.rank))
                .filter(|(uid, _)| uid == &person.id)
                .next()
                .map(|(_, rank)| rank)
        } else {
            None
        };

        Ok(ListBoardModsResponse {
            mods,
            pending_mods,
            has_pending_invite,
            my_mod_rank,
        })
    }
}
