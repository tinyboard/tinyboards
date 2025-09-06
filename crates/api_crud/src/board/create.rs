use crate::PerformCrud;
use actix_web::web::Data;
use regex::Regex;
use tinyboards_api_common::{
    board::{BoardExistsResponse, BoardResponse, CheckBoardExists, CreateBoard},
    build_response::build_board_response,
    data::TinyBoardsContext,
    utils::{
        generate_featured_url, generate_inbox_url, generate_local_apub_endpoint,
        generate_moderators_url, generate_shared_inbox_url, generate_subscribers_url, require_user,
        EndpointType,
    },
};
use tinyboards_db::models::site::local_site::LocalSite;
use tinyboards_db::{
    models::{
        board::{
            board_mods::{BoardModerator, BoardModeratorForm, ModPerms},
            board_subscriber::{BoardSubscriber, BoardSubscriberForm},
            boards::{Board, BoardForm},
        },
        person::local_user::AdminPerms,
    },
    traits::{ApubActor, Crud, Joinable, Subscribeable},
};
use tinyboards_db_views::structs::SiteView;
use tinyboards_utils::generate_actor_keypair;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for CheckBoardExists {
    type Response = BoardExistsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data = &self;
        let existence = Board::board_exists(context.pool(), &data.board_name)
            .await
            .map_err(|_| TinyBoardsError::from_message(500, "Server Error while checking board"))?;

        Ok(BoardExistsResponse { result: existence })
    }
}

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

        let local_site = LocalSite::read(context.pool()).await?;

        let name = data.name.clone();
        let title = data.title.clone();
        let description = data.description.clone();
        let primary_color = data.primary_color.clone().unwrap_or(
            local_site
                .primary_color
                .unwrap_or("60, 105, 145".to_string()),
        );
        let secondary_color = data.secondary_color.clone().unwrap_or(
            local_site
                .secondary_color
                .unwrap_or("96, 128, 63".to_string()),
        );
        let hover_color = data
            .hover_color
            .clone()
            .unwrap_or(local_site.hover_color.unwrap_or("54, 94, 129".to_string()));

        let mut view = require_user(context.pool(), context.master_key(), auth).await;

        if local_site.board_creation_admin_only {
            view = view.require_admin(AdminPerms::Boards);
        }

        let view = view.unwrap()?;

        // Check name
        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{1,29}$").unwrap();
        if !re.is_match(&name) {
            return Err(TinyBoardsError::from_message(400, "Board name contains disallowed characters. Allowed: alphanumerics and underscores, except as the first character."));
        }

        let site_view = SiteView::read_local(context.pool()).await?;

        /*if let Some(desc) = description {
            description = parse_markdown_opt(&desc);
        }*/

        let icon = &data.icon;
        let banner = &data.banner;

        let board_actor_id = generate_local_apub_endpoint(
            EndpointType::Board,
            &data.name,
            &context.settings().get_protocol_and_hostname(),
        )?;

        // check for dupe actor id
        let board_dupe = Board::read_from_apub_id(context.pool(), &board_actor_id).await?;

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
            primary_color: Some(primary_color),
            secondary_color: Some(secondary_color),
            hover_color: Some(hover_color),
            ..BoardForm::default()
        };

        // create the board
        let board = Board::create(context.pool(), &form).await?;

        // the board creator becomes a board mod
        let board_mod_form = BoardModeratorForm {
            board_id: Some(board.id),
            person_id: Some(view.person.id),
            rank: Some(1),
            permissions: Some(ModPerms::Full.as_i32()),
            invite_accepted: Some(true),
            invite_accepted_date: None,
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
