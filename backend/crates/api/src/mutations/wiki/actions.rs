use async_graphql::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        wiki::{
            WikiPage as DbWikiPage, WikiPageRevision as DbWikiPageRevision,
            WikiPageRevisionInsertForm, WikiPageUpdateForm,
        },
    },
    schema::{board_moderators, wiki_page_revisions, wiki_pages},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    structs::wiki::{EditWikiPageInput, WikiPage},
    LoggedInUser,
};

/// Check if user has wiki mod permissions for a board
async fn require_wiki_mod(
    conn: &mut diesel_async::AsyncPgConnection,
    user: &tinyboards_db::models::user::user::User,
    board_id: Uuid,
) -> Result<()> {
    if user.is_admin {
        return Ok(());
    }

    let moderator: BoardModerator = board_moderators::table
        .filter(board_moderators::board_id.eq(board_id))
        .filter(board_moderators::user_id.eq(user.id))
        .filter(board_moderators::is_invite_accepted.eq(true))
        .first(conn)
        .await
        .map_err(|_| TinyBoardsError::from_message(403, "Only moderators can perform this action"))?;

    if !moderator.has_permission(ModPerms::Wiki) {
        return Err(TinyBoardsError::from_message(403, "Insufficient wiki permissions").into());
    }
    Ok(())
}

#[derive(Default)]
pub struct WikiPageActions;

#[Object]
impl WikiPageActions {
    /// Edit a wiki page (creates a new revision)
    async fn edit_wiki_page(
        &self,
        ctx: &Context<'_>,
        page_id: ID,
        input: EditWikiPageInput,
    ) -> Result<WikiPage> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let page_uuid: Uuid = page_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid page ID".into()))?;

        let page: DbWikiPage = wiki_pages::table
            .find(page_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Wiki page not found".into()))?;

        // Check permissions
        require_wiki_mod(conn, user, page.board_id).await?;

        if page.is_locked && !user.is_admin {
            return Err(
                TinyBoardsError::from_message(403, "This page is locked").into(),
            );
        }

        // Get current revision count
        let current_rev: i64 = wiki_page_revisions::table
            .filter(wiki_page_revisions::page_id.eq(page_uuid))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let new_body = input.body.clone().unwrap_or_else(|| page.body.clone());
        let new_body_html = input.body.unwrap_or_else(|| page.body_html.clone());

        // Create revision
        let revision_form = WikiPageRevisionInsertForm {
            page_id: page_uuid,
            revision_number: (current_rev + 1) as i32,
            editor_id: user.id,
            edit_summary: input.edit_summary,
            body: new_body.clone(),
            body_html: new_body_html.clone(),
        };

        diesel::insert_into(wiki_page_revisions::table)
            .values(&revision_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Update the page
        let update_form = WikiPageUpdateForm {
            slug: None,
            title: input.title,
            body: Some(new_body),
            body_html: Some(new_body_html),
            last_edited_by: Some(Some(user.id)),
            view_permission: None,
            edit_permission: None,
            is_locked: input.is_locked,
            display_order: input.display_order.map(Some),
            parent_id: None,
            deleted_at: None,
        };

        let updated: DbWikiPage = diesel::update(wiki_pages::table.find(page_uuid))
            .set(&update_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(WikiPage::from(updated))
    }

    /// Soft delete a wiki page
    async fn delete_wiki_page(&self, ctx: &Context<'_>, page_id: ID) -> Result<bool> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let page_uuid: Uuid = page_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid page ID".into()))?;

        let page: DbWikiPage = wiki_pages::table
            .find(page_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Wiki page not found".into()))?;

        require_wiki_mod(conn, user, page.board_id).await?;

        let updated = diesel::update(wiki_pages::table.find(page_uuid))
            .set(wiki_pages::deleted_at.eq(Some(Utc::now())))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(updated > 0)
    }

    /// Revert a wiki page to a previous revision
    async fn revert_wiki_page(
        &self,
        ctx: &Context<'_>,
        page_id: ID,
        revision_number: i32,
    ) -> Result<WikiPage> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let conn = &mut get_conn(pool).await?;

        let page_uuid: Uuid = page_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid page ID".into()))?;

        let page: DbWikiPage = wiki_pages::table
            .find(page_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Wiki page not found".into()))?;

        require_wiki_mod(conn, user, page.board_id).await?;

        // Get the target revision
        let target_rev: DbWikiPageRevision = wiki_page_revisions::table
            .filter(wiki_page_revisions::page_id.eq(page_uuid))
            .filter(wiki_page_revisions::revision_number.eq(revision_number))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Revision not found".into()))?;

        // Get current revision count
        let current_rev: i64 = wiki_page_revisions::table
            .filter(wiki_page_revisions::page_id.eq(page_uuid))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Create a new revision that reverts to the target
        let revert_form = WikiPageRevisionInsertForm {
            page_id: page_uuid,
            revision_number: (current_rev + 1) as i32,
            editor_id: user.id,
            edit_summary: Some(format!("Reverted to revision {}", revision_number)),
            body: target_rev.body.clone(),
            body_html: target_rev.body_html.clone(),
        };

        diesel::insert_into(wiki_page_revisions::table)
            .values(&revert_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Update page content
        let update_form = WikiPageUpdateForm {
            slug: None,
            title: None,
            body: Some(target_rev.body),
            body_html: Some(target_rev.body_html),
            last_edited_by: Some(Some(user.id)),
            view_permission: None,
            edit_permission: None,
            is_locked: None,
            display_order: None,
            parent_id: None,
            deleted_at: None,
        };

        let updated: DbWikiPage = diesel::update(wiki_pages::table.find(page_uuid))
            .set(&update_form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(WikiPage::from(updated))
    }
}
