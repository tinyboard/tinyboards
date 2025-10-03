use crate::{LoggedInUser, structs::boards::Board, helpers::files::upload::upload_file_opendal};
use async_graphql::*;
use tinyboards_db::{
    models::board::{
        boards::{Board as DbBoard, BoardForm},
        board_mods::{BoardModerator, BoardModeratorForm},
        board_subscriber::{BoardSubscriber, BoardSubscriberForm},
    },
    models::site::site::Site,
    traits::{Crud, Subscribeable, Joinable},
    utils::DbPool,
};
use url::Url;
use tinyboards_utils::TinyBoardsError;
use chrono::Utc;

#[derive(InputObject)]
pub struct CreateBoardInput {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub is_nsfw: Option<bool>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub hover_color: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
}

#[derive(SimpleObject)]
pub struct CreateBoardResponse {
    pub board: Board,
}

#[derive(Default)]
pub struct CreateBoard;

#[Object]
impl CreateBoard {
    /// Create a new board
    async fn create_board(
        &self,
        ctx: &Context<'_>,
        input: CreateBoardInput,
        icon_file: Option<Upload>,
        banner_file: Option<Upload>,
    ) -> Result<CreateBoardResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Check if board creation is allowed
        let site = Site::read(pool).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read site settings"))?;

        // Check if user can create boards based on board creation mode
        let admin_level = user.admin_level;

        match site.board_creation_mode.as_str() {
            "Disabled" => {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Board creation is currently disabled"
                ).into());
            }
            "AdminOnly" => {
                if admin_level == 0 {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Board creation is restricted to admins only"
                    ).into());
                }
            }
            "TrustedUsers" => {
                // Admins bypass all checks
                if admin_level == 0 {
                    // Check manual approval if required
                    if site.trusted_user_manual_approval && !user.board_creation_approved {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "Board creation requires manual approval from an administrator. Please contact an admin to request approval."
                        ).into());
                    }

                    // Check automatic requirements
                    use tinyboards_db::aggregates::structs::UserAggregates;
                    let user_aggregates = UserAggregates::read(pool, user.id).await
                        .map_err(|e| TinyBoardsError::from_error_message(
                            e,
                            500,
                            "Failed to read user statistics"
                        ))?;

                    let user_reputation = user_aggregates.post_score + user_aggregates.comment_score;

                    // Calculate account age in days
                    let account_age_days = (Utc::now().naive_utc() - user.creation_date).num_days();

                    // Check reputation requirement
                    if user_reputation < site.trusted_user_min_reputation as i64 {
                        return Err(TinyBoardsError::from_message(
                            403,
                            &format!(
                                "Insufficient reputation to create boards. Required: {} points, You have: {} points",
                                site.trusted_user_min_reputation,
                                user_reputation
                            )
                        ).into());
                    }

                    // Check account age requirement
                    if account_age_days < site.trusted_user_min_account_age_days as i64 {
                        return Err(TinyBoardsError::from_message(
                            403,
                            &format!(
                                "Account too new to create boards. Required: {} days, Your account age: {} days",
                                site.trusted_user_min_account_age_days,
                                account_age_days
                            )
                        ).into());
                    }

                    // Check minimum posts requirement
                    if user_aggregates.post_count < site.trusted_user_min_posts as i64 {
                        return Err(TinyBoardsError::from_message(
                            403,
                            &format!(
                                "Insufficient posts to create boards. Required: {} posts, You have: {} posts",
                                site.trusted_user_min_posts,
                                user_aggregates.post_count
                            )
                        ).into());
                    }
                }
            }
            "Open" | _ => {
                // Allow anyone to create boards (no restrictions)
            }
        }

        // Validate board name - basic validation for now
        if input.name.is_empty() || input.name.len() > 50 || input.name.contains(' ') {
            return Err(TinyBoardsError::from_message(400, "Invalid board name").into());
        }

        // Check if board name already exists
        if DbBoard::get_by_name(pool, &input.name).await.is_ok() {
            return Err(TinyBoardsError::from_message(400, "Board name already exists").into());
        }

        // Handle file uploads
        let icon_url = match icon_file {
            Some(file) => Some(upload_file_opendal(file, None, user.id, Some(2), ctx).await?.into()),
            None => input.icon.and_then(|url_str| {
                Url::parse(&url_str).ok().map(|url| url.into())
            }),
        };

        let banner_url = match banner_file {
            Some(file) => Some(upload_file_opendal(file, None, user.id, Some(5), ctx).await?.into()),
            None => input.banner.and_then(|url_str| {
                Url::parse(&url_str).ok().map(|url| url.into())
            }),
        };

        let board_form = BoardForm {
            name: Some(input.name),
            title: Some(input.title),
            description: Some(input.description),
            is_nsfw: input.is_nsfw,
            primary_color: input.primary_color.or(Some("#1976d2".to_string())),
            secondary_color: input.secondary_color.or(Some("#424242".to_string())),
            hover_color: input.hover_color.or(Some("#1565c0".to_string())),
            icon: icon_url,
            banner: banner_url,
            last_refreshed_date: Some(Utc::now().naive_utc()),
            ..Default::default()
        };

        // Create the board
        let db_board = DbBoard::create(pool, &board_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create board"))?;

        // Add creator as moderator
        let mod_form = BoardModeratorForm {
            board_id: Some(db_board.id),
            user_id: Some(user.id),
            permissions: Some(8191), // All permissions
            rank: Some(1), // Top rank
            invite_accepted: Some(true),
            invite_accepted_date: Some(Some(Utc::now().naive_utc())),
        };

        BoardModerator::join(pool, &mod_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to add creator as moderator"))?;

        // Subscribe creator to the board
        let sub_form = BoardSubscriberForm {
            board_id: db_board.id,
            user_id: user.id,
            pending: Some(false),
        };

        BoardSubscriber::subscribe(pool, &sub_form).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to subscribe creator to board"))?;

        // Get board with counts for the GraphQL response
        let board_with_counts = DbBoard::get_with_counts_for_ids(pool, vec![db_board.id]).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read board with counts"))?
            .into_iter()
            .next()
            .ok_or_else(|| TinyBoardsError::from_message(500, "Failed to get created board with counts"))?;

        let board = Board::from(board_with_counts);

        Ok(CreateBoardResponse { board })
    }
}