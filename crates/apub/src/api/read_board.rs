use crate::{
    api::PerformApub,
    fetcher::resolve_actor_identifier,
    objects::board::ApubBoard,
  };
  use tinyboards_federation::config::Data;
  use tinyboards_api_common::{
    board::{GetBoard, GetBoardResponse},
    data::TinyBoardsContext,
    utils::{check_private_instance, require_user_opt, is_mod_or_admin_opt},
  };
  use tinyboards_db::models::{
    apub::actor_language::BoardLanguage,
    board::boards::Board,
    site::site::Site,
  };
  use tinyboards_db_views::structs::{BoardModeratorView, BoardView};
  use tinyboards_utils::error::TinyBoardsError;

  #[async_trait::async_trait]
  impl PerformApub for GetBoard {
    type Response = GetBoardResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(&self, context: &Data<TinyBoardsContext>, auth: Option<&str>) -> Result<GetBoardResponse, TinyBoardsError> {
        let data: &GetBoard = self;

        let view = require_user_opt(context.pool(), context.master_key(), auth).await?;
        
        if data.name.is_none() && data.id.is_none() {
            return Err(TinyBoardsError::from_message(400, "no id or name given."));
        }

        check_private_instance(&view.clone().map(|u| u.local_user), context.pool()).await?;

        let person_id = view.as_ref().map(|u| u.person.id);

        let board_id = match data.id {
            Some(id) => id,
            None => {
                let name = data.name.clone().unwrap_or_else(|| "campfire".to_string());
                resolve_actor_identifier::<ApubBoard, Board>(&name, context, &view, true)
                    .await?
                    .id
            }
        };

        let is_mod_or_admin = 
            is_mod_or_admin_opt(context.pool(), view.as_ref(), Some(board_id))
                .await
                .is_ok();

        let board_view = BoardView::read(
            context.pool(),
            board_id,
            person_id,
            Some(is_mod_or_admin)
        )
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't find board."))?;

        let moderators = BoardModeratorView::for_board(context.pool(), board_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't find board"))?;


        let site_id = 
            Site::instance_actor_id_from_url(board_view.board.actor_id.clone().into());
        
        let mut site = Site::read_from_apub_id(context.pool(), &site_id.into()).await?;

        // no need to include metadata for local site
        // prevents federation private key leakage too
        if let Some(s) = &site {
            if s.actor_id.domain() == Some(context.settings().hostname.as_ref()) {
                site = None;
            }
        }

        let board_id = board_view.board.id;
        let discussion_languages = BoardLanguage::read(context.pool(), board_id).await?;

        let res = GetBoardResponse {
            board_view,
            site,
            moderators,
            discussion_languages,
        };

        Ok(res)
    }
}