use crate::{
    helpers::files::upload::upload_file_opendal,
    structs::boards::Board,
};
use async_graphql::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::{DbBoardMode, DbWikiPermission},
    models::{
        aggregates::BoardAggregates as DbBoardAggregates,
        board::{
            board_mods::BoardModeratorInsertForm,
            boards::{Board as DbBoard, BoardInsertForm},
        },
        site::site::Site as DbSite,
    },
    schema::{board_aggregates, board_moderators, board_subscribers, boards, site, user_aggregates},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use url::Url;
use uuid::Uuid;

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
    /// Board mode: "feed" or "forum". Defaults to site.default_board_mode if omitted.
    pub mode: Option<String>,
    /// Whether to enable the wiki for this board.
    pub wiki_enabled: Option<bool>,
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
        let user = ctx
            .data::<crate::LoggedInUser>()?
            .require_user_approved(pool)
            .await?;

        // Load site configuration to enforce board creation mode
        let site: DbSite = {
            let conn = &mut get_conn(pool).await?;
            site::table
                .first(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(format!("Failed to read site settings: {}", e)))?
        };

        let admin_level = user.admin_level;

        // Normalize to lowercase for case-insensitive matching
        match site.board_creation_mode.to_lowercase().as_str() {
            "disabled" => {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Board creation is currently disabled",
                )
                .into());
            }
            "adminonly" | "admin_only" | "closed" => {
                if admin_level == 0 {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Board creation is restricted to admins only",
                    )
                    .into());
                }
            }
            "trustedusers" | "trusted_users" | "restricted" => {
                // Admins bypass all checks
                if admin_level == 0 {
                    // Check manual approval if required
                    if site.trusted_user_manual_approval && !user.is_board_creation_approved {
                        return Err(TinyBoardsError::from_message(
                            403,
                            "Board creation requires manual approval from an administrator. Please contact an admin to request approval.",
                        )
                        .into());
                    }

                    // Check automatic requirements via user aggregates
                    use tinyboards_db::models::aggregates::UserAggregates;

                    let user_agg_opt: Option<UserAggregates> = {
                        let conn = &mut get_conn(pool).await?;
                        user_aggregates::table
                            .filter(user_aggregates::user_id.eq(user.id))
                            .first(conn)
                            .await
                            .optional()
                            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
                    };

                    let user_agg = user_agg_opt.unwrap_or_else(|| UserAggregates {
                        id: Uuid::nil(),
                        user_id: user.id,
                        post_count: 0,
                        post_score: 0,
                        comment_count: 0,
                        comment_score: 0,
                        created_at: Utc::now(),
                    });

                    let user_reputation = user_agg.post_score + user_agg.comment_score;
                    let account_age_days = (Utc::now() - user.created_at).num_days();

                    if user_reputation < site.trusted_user_min_reputation as i64 {
                        return Err(TinyBoardsError::from_message(
                            403,
                            &format!(
                                "Insufficient reputation to create boards. Required: {} points, You have: {} points",
                                site.trusted_user_min_reputation, user_reputation
                            ),
                        )
                        .into());
                    }

                    if account_age_days < site.trusted_user_min_account_age_days as i64 {
                        return Err(TinyBoardsError::from_message(
                            403,
                            &format!(
                                "Account too new to create boards. Required: {} days, Your account age: {} days",
                                site.trusted_user_min_account_age_days, account_age_days
                            ),
                        )
                        .into());
                    }

                    if user_agg.post_count < site.trusted_user_min_posts as i64 {
                        return Err(TinyBoardsError::from_message(
                            403,
                            &format!(
                                "Insufficient posts to create boards. Required: {} posts, You have: {} posts",
                                site.trusted_user_min_posts, user_agg.post_count
                            ),
                        )
                        .into());
                    }
                }
            }
            // "open" or any unrecognised value: no restrictions
            _ => {
                // Also check the legacy board_creation_admin_only flag
                if site.board_creation_admin_only && admin_level == 0 {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Board creation is restricted to admins only",
                    )
                    .into());
                }
            }
        }

        // Validate board name
        if input.name.is_empty() || input.name.len() > 50 || input.name.contains(' ') {
            return Err(TinyBoardsError::from_message(400, "Invalid board name").into());
        }

        // Check if board name already exists
        let name_taken: bool = {
            let conn = &mut get_conn(pool).await?;
            let existing: Option<DbBoard> = boards::table
                .filter(boards::name.eq(&input.name))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
            existing.is_some()
        };

        if name_taken {
            return Err(TinyBoardsError::from_message(400, "Board name already exists").into());
        }

        // Handle file uploads
        let icon_url = match icon_file {
            Some(file) => Some(
                upload_file_opendal(file, None, user.id, Some(2), ctx)
                    .await?
                    .to_string(),
            ),
            None => input.icon,
        };

        let banner_url = match banner_file {
            Some(file) => Some(
                upload_file_opendal(file, None, user.id, Some(5), ctx)
                    .await?
                    .to_string(),
            ),
            None => input.banner,
        };

        let icon_stored = icon_url.and_then(|s| Url::parse(&s).ok().map(|u| u.to_string()));
        let banner_stored = banner_url.and_then(|s| Url::parse(&s).ok().map(|u| u.to_string()));

        // Determine board mode: use caller's choice, or fall back to site default
        let board_mode = match input.mode.as_deref() {
            Some("forum") => DbBoardMode::Forum,
            Some("feed") => DbBoardMode::Feed,
            Some(other) => {
                return Err(TinyBoardsError::from_message(
                    400,
                    &format!("Invalid board mode '{}'. Must be 'feed' or 'forum'.", other),
                )
                .into());
            }
            None => site.default_board_mode,
        };

        let board_form = BoardInsertForm {
            name: input.name,
            title: input.title,
            description: input.description,
            sidebar: None,
            sidebar_html: None,
            icon: icon_stored,
            banner: banner_stored,
            primary_color: input
                .primary_color
                .unwrap_or_else(|| "#1976d2".to_string()),
            secondary_color: input
                .secondary_color
                .unwrap_or_else(|| "#424242".to_string()),
            hover_color: input
                .hover_color
                .unwrap_or_else(|| "#1565c0".to_string()),
            is_nsfw: input.is_nsfw.unwrap_or(false),
            is_hidden: false,
            is_removed: false,
            is_banned: false,
            is_posting_restricted_to_mods: false,
            exclude_from_all: false,
            ban_reason: None,
            public_ban_reason: None,
            banned_by: None,
            banned_at: None,
            mode: board_mode,
            wiki_enabled: input.wiki_enabled.unwrap_or(false),
            wiki_require_approval: None,
            wiki_default_view_permission: DbWikiPermission::Public,
            wiki_default_edit_permission: DbWikiPermission::ModsOnly,
            custom_css: None,
        };

        // All remaining DB work uses one connection
        let conn = &mut get_conn(pool).await?;

        // Insert the board
        let db_board: DbBoard = diesel::insert_into(boards::table)
            .values(&board_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to create board: {}", e)))?;

        // Add creator as moderator with full permissions at rank 0 (owner)
        let mod_form = BoardModeratorInsertForm {
            board_id: db_board.id,
            user_id: user.id,
            permissions: 0x7FFFFFFF, // Full permissions bitmask
            rank: 0,
            is_invite_accepted: true,
            invite_accepted_at: Some(Utc::now()),
        };

        diesel::insert_into(board_moderators::table)
            .values(&mod_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to add creator as moderator: {}", e)))?;

        // Subscribe creator to the board
        diesel::insert_into(board_subscribers::table)
            .values((
                board_subscribers::board_id.eq(db_board.id),
                board_subscribers::user_id.eq(user.id),
                board_subscribers::is_pending.eq(false),
            ))
            .on_conflict((board_subscribers::board_id, board_subscribers::user_id))
            .do_nothing()
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(format!("Failed to subscribe creator to board: {}", e)))?;

        // Aggregates row may not exist yet for a brand-new board; return None if missing
        let agg: Option<DbBoardAggregates> = board_aggregates::table
            .filter(board_aggregates::board_id.eq(db_board.id))
            .first(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let board = Board::from_db(db_board, agg);

        Ok(CreateBoardResponse { board })
    }
}
