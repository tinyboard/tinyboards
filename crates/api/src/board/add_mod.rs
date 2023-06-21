use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{AddBoardMod, AddBoardModResponse},
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, BoardModeratorForm},
        moderator::mod_actions::{ModAddBoardMod, ModAddBoardModForm},
    },
    traits::Crud,
};
use tinyboards_db_views::structs::BoardModeratorView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for AddBoardMod {
    type Response = AddBoardModResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &AddBoardMod = &self;

        // require admin to add board moderator
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let added = data.added;
        let person_id = data.person_id;
        let board_id = data.board_id;

        // board moderator form (for adding or removing mod status)
        let form = BoardModeratorForm {
            board_id,
            person_id,
        };

        if added {
            // add board moderator status for the targeted user on the targeted board
            BoardModerator::create(context.pool(), &form).await?;
        } else {
            // remove board moderator status for the targeted user on the targeted board
            BoardModerator::remove_board_mod(context.pool(), &form).await?;
        }

        // log this mod action
        let mod_add_board_mod_form = ModAddBoardModForm {
            mod_person_id: view.person.id,
            other_person_id: person_id.clone(),
            removed: Some(Some(!added.clone())),
            board_id: board_id.clone(),
        };

        // submit to the mod log
        ModAddBoardMod::create(context.pool(), &mod_add_board_mod_form).await?;

        let board_id = data.board_id;
        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(AddBoardModResponse { moderators })
    }
}
