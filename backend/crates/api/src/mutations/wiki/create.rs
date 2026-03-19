use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::DbWikiPermission,
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        wiki::{WikiPage as DbWikiPage, WikiPageInsertForm},
    },
    schema::{board_moderators, boards, wiki_pages},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::wiki::{CreateWikiPageInput, WikiPage},
    LoggedInUser,
};

#[derive(Default)]
pub struct CreateWikiPage;

#[Object]
impl CreateWikiPage {
    /// Create a new wiki page for a board
    async fn create_wiki_page(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        input: CreateWikiPageInput,
    ) -> Result<WikiPage> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid board ID".into()))?;

        // Verify board exists
        let _board_name: String = boards::table
            .find(board_uuid)
            .select(boards::name)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".into()))?;

        // Check mod permissions
        if !user.is_admin {
            let moderator: BoardModerator = board_moderators::table
                .filter(board_moderators::board_id.eq(board_uuid))
                .filter(board_moderators::user_id.eq(user.id))
                .filter(board_moderators::is_invite_accepted.eq(true))
                .first(conn)
                .await
                .map_err(|_| {
                    TinyBoardsError::from_message(403, "Only moderators can create wiki pages")
                })?;

            if !moderator.has_permission(ModPerms::Wiki) {
                return Err(TinyBoardsError::from_message(
                    403,
                    "Insufficient wiki permissions",
                )
                .into());
            }
        }

        // Check for duplicate slug
        let existing_count: i64 = wiki_pages::table
            .filter(wiki_pages::board_id.eq(board_uuid))
            .filter(wiki_pages::slug.eq(&input.slug))
            .filter(wiki_pages::deleted_at.is_null())
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if existing_count > 0 {
            return Err(TinyBoardsError::from_message(
                409,
                "A wiki page with this slug already exists",
            )
            .into());
        }

        let view_permission = parse_wiki_permission(
            input.view_permission.as_deref().unwrap_or("public"),
        )?;
        let edit_permission = parse_wiki_permission(
            input.edit_permission.as_deref().unwrap_or("mods_only"),
        )?;

        let parent_uuid: Option<Uuid> = if let Some(ref pid) = input.parent_id {
            Some(
                pid.parse()
                    .map_err(|_| TinyBoardsError::NotFound("Invalid parent page ID".into()))?,
            )
        } else {
            None
        };

        // For now, use the body as both body and body_html (HTML processing can be added later)
        let body_html = input.body.clone();

        let form = WikiPageInsertForm {
            board_id: board_uuid,
            slug: input.slug,
            title: input.title,
            body: input.body,
            body_html,
            creator_id: user.id,
            last_edited_by: None,
            view_permission,
            edit_permission,
            is_locked: false,
            display_order: input.display_order,
            parent_id: parent_uuid,
        };

        let page: DbWikiPage = diesel::insert_into(wiki_pages::table)
            .values(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(WikiPage::from(page))
    }
}

fn parse_wiki_permission(s: &str) -> Result<DbWikiPermission> {
    match s {
        "public" => Ok(DbWikiPermission::Public),
        "members" => Ok(DbWikiPermission::Members),
        "mods_only" | "mods" => Ok(DbWikiPermission::ModsOnly),
        "locked" => Ok(DbWikiPermission::Locked),
        _ => Err(TinyBoardsError::from_message(400, "Invalid wiki permission value").into()),
    }
}
