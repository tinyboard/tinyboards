use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    board::{CreateBoard, BoardResponse},
    utils::{
        require_user, generate_local_apub_endpoint, EndpointType, generate_subscribers_url, generate_inbox_url, generate_featured_url, generate_moderators_url, generate_shared_inbox_url,
    }, build_response::build_board_response,
};
use tinyboards_db::{
    models::{
        board::boards::{Board, BoardForm}, 
        board::board_mods::{BoardModeratorForm, BoardModerator}, 
        board::board_subscriber::{BoardSubscriber, BoardSubscriberForm},
    },
    traits::{Crud, ApubActor, Subscribeable, Joinable},
};
use tinyboards_db_views::structs::SiteView;
use tinyboards_federation::http_signatures::generate_actor_keypair;
use tinyboards_utils::{
    parser::parse_markdown_opt,
    TinyBoardsError,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CreateBoard {
    type Response = BoardResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<BoardResponse, TinyBoardsError> {
        let data: &CreateBoard = &self;

        let name = data.name.clone();
        let title = data.title.clone();
        let mut description = data.description.clone();


        // board creation restricted to admins (may provide other options in the future)
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let site_view = SiteView::read_local(context.pool()).await?;

        if let Some(desc) = description {
            description = parse_markdown_opt(&desc);
        }

        let icon = &data.icon;
        let banner = &data.banner;

        let board_actor_id = generate_local_apub_endpoint(
            EndpointType::Board, 
            &data.name, 
            &context.settings().get_protocol_and_hostname(),
        )?;

        // check for dupe actor id
        let board_dupe = Board::read_from_apub_id(
            context.pool(), 
            &board_actor_id
        )
        .await?;
        
        if board_dupe.is_some() {
            return Err(TinyBoardsError::from_message(400, "board already exists."));
        }

        let keypair = generate_actor_keypair()?;

        let form = BoardForm {
            name: Some(name),
            title: Some(title),
            description: Some(description),
            icon: icon.clone(),
            banner: banner.clone(),
            private_key: Some(keypair.private_key),
            public_key: Some(keypair.public_key),
            subscribers_url: Some(generate_subscribers_url(&board_actor_id)?),
            inbox_url: Some(generate_inbox_url(&board_actor_id)?),
            shared_inbox_url: Some(Some(generate_shared_inbox_url(&board_actor_id)?)),
            featured_url: Some(generate_featured_url(&board_actor_id)?),
            moderators_url: Some(generate_moderators_url(&board_actor_id)?),
            instance_id: Some(site_view.site.instance_id),            
            ..BoardForm::default()
        };

        // create the board
        let board = Board::create(context.pool(), &form).await?;

        // the board creator becomes a board mod
        let board_mod_form = BoardModeratorForm {
            board_id: board.id,
            person_id: view.person.id,
        };
        BoardModerator::join(context.pool(), &board_mod_form).await?;

        // subscribe to your own board
        let subscribe_form = BoardSubscriberForm {
            board_id: board.id,
            person_id: view.person.id,
            pending: Some(false),
        };
        BoardSubscriber::subscribe(context.pool(), &subscribe_form).await?;

        // TODO: logic about updating the discussion languages of the board if provided

        Ok(build_board_response(context, view, board.id).await?)
    }
}