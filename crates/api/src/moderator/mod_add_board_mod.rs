use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{AddBoardMod, ModActionResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, BoardModeratorForm},
        moderator::mod_actions::{ModAddBoardMod, ModAddBoardModForm},
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for AddBoardMod {
    type Response = ModActionResponse<ModAddBoardMod>;
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
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let added = data.added;
        let added_user_id = data.added_user_id;
        let added_board_id = data.added_board_id;

        // board moderator form (for adding or removing mod status)
        let form = BoardModeratorForm {
            board_id: added_board_id.clone(),
            user_id: added_board_id.clone(),
        };

        if added {
            // add board moderator status for the targeted user on the targeted board
            blocking(context.pool(), move |conn| {
                BoardModerator::create(conn, &form)
            })
            .await??;
        } else {
            // remove board moderator status for the targeted user on the targeted board
            blocking(context.pool(), move |conn| {
                BoardModerator::remove_board_mod(conn, &form)
            })
            .await??;
        }

        // log this mod action
        let mod_add_board_mod_form = ModAddBoardModForm {
            mod_user_id: user.id,
            other_user_id: added_user_id.clone(),
            removed: Some(Some(!added.clone())),
            board_id: added_board_id.clone(),
        };

        // submit to the mod log
        let mod_action = blocking(context.pool(), move |conn| {
            ModAddBoardMod::create(conn, &mod_add_board_mod_form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}
