use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    board::{AddBoardMod, BoardIdPath, BoardModResponse},
    data::TinyBoardsContext,
    utils::require_user,
};
use tinyboards_db::{
    models::{
        board::{
            board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
            boards::Board,
        },
        person::local_user::AdminPerms,
    },
    traits::Crud,
};
use tinyboards_db_views::structs::BoardModeratorView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for AddBoardMod {
    type Response = BoardModResponse;
    type Route = BoardIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        BoardIdPath { board_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &AddBoardMod = &self;
        //let board_id = data.board_id;

        if board_id == 1 {
            return Err(TinyBoardsError::from_message(400, "You cannot directly add a mod to the default board. Users become mods of the default board automatically when they're made admins with at least the Manage Content permission."));
        }

        // check permissions later
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            //.require_board_mod(context.pool(), board_id, ModPerms::Full)
            .unwrap()?;

        let person_id = data.person_id.unwrap_or(view.person.id);

        if !(person_id == view.person.id || view.local_user.has_permission(AdminPerms::Boards)) {
            return Err(TinyBoardsError::from_message(
                403,
                "You cannot just add a mod like that...",
            ));
        }

        // check if there is a mod invite to the user
        let mod_invite = Board::get_mod_invite(context.pool(), board_id, person_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Something went wrong while checking what you're trying to do. Sorry about that."))?;

        // if the invite exists, we only have to accept it by updating an existing mod relationship
        match mod_invite {
            Some(mod_invite) => {
                mod_invite
                    .accept_invite(context.pool())
                    .await
                    .map_err(|e| {
                        TinyBoardsError::from_error_message(
                            e,
                            500,
                            "Couldn't accept invite due to some ficky-fucky.",
                        )
                    })?;
            }
            None => {
                // This is what admins use to appoint themselves as mods. No normies allowed here!!
                if !view.local_user.has_permission(AdminPerms::Boards) {
                    return Err(TinyBoardsError::from_message(403, "You haven't been invited to become a mod, or you were too late and your invite was revoked. L."));
                }

                let form = BoardModeratorForm {
                    board_id: Some(board_id),
                    person_id: Some(person_id),
                    invite_accepted: Some(true),
                    permissions: Some(ModPerms::Full.as_i32()),
                    ..BoardModeratorForm::default()
                };

                BoardModerator::create(context.pool(), &form).await?;
            }
        }

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        Ok(BoardModResponse { moderators })
    }
}
