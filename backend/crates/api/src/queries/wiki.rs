use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::wiki::{WikiPage as DbWikiPage, WikiPageRevision as DbWikiPageRevision},
    schema::{boards, wiki_page_revisions, wiki_pages},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::structs::wiki::{WikiPage, WikiPageRevision};

#[derive(Default)]
pub struct QueryWiki;

#[Object]
impl QueryWiki {
    /// Get a specific wiki page by board name and slug
    async fn wiki_page(
        &self,
        ctx: &Context<'_>,
        board_name: String,
        slug: String,
    ) -> Result<WikiPage> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        // Get board ID by name
        let board_id: Uuid = boards::table
            .filter(boards::name.eq(&board_name))
            .select(boards::id)
            .first(conn)
            .await
            .map_err(|_| {
                TinyBoardsError::NotFound(format!("Board '{}' not found", board_name))
            })?;

        // Get wiki page
        let page: DbWikiPage = wiki_pages::table
            .filter(wiki_pages::board_id.eq(board_id))
            .filter(wiki_pages::slug.eq(&slug))
            .filter(wiki_pages::deleted_at.is_null())
            .first(conn)
            .await
            .map_err(|_| {
                TinyBoardsError::NotFound(format!("Wiki page '{}' not found", slug))
            })?;

        Ok(WikiPage::from(page))
    }

    /// List all wiki pages for a board
    async fn list_wiki_pages(
        &self,
        ctx: &Context<'_>,
        board_name: String,
        #[graphql(default = false)] include_deleted: bool,
    ) -> Result<Vec<WikiPage>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let board_id: Uuid = boards::table
            .filter(boards::name.eq(&board_name))
            .select(boards::id)
            .first(conn)
            .await
            .map_err(|_| {
                TinyBoardsError::NotFound(format!("Board '{}' not found", board_name))
            })?;

        let mut query = wiki_pages::table
            .filter(wiki_pages::board_id.eq(board_id))
            .into_boxed();

        if !include_deleted {
            query = query.filter(wiki_pages::deleted_at.is_null());
        }

        let pages: Vec<DbWikiPage> = query
            .order(wiki_pages::display_order.asc().nulls_last())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(pages.into_iter().map(WikiPage::from).collect())
    }

    /// Get revision history for a wiki page
    async fn wiki_page_history(
        &self,
        ctx: &Context<'_>,
        page_id: ID,
    ) -> Result<Vec<WikiPageRevision>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let page_uuid: Uuid = page_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid page ID".into()))?;

        // Verify page exists
        let _page: DbWikiPage = wiki_pages::table
            .find(page_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Wiki page not found".into()))?;

        let revisions: Vec<DbWikiPageRevision> = wiki_page_revisions::table
            .filter(wiki_page_revisions::page_id.eq(page_uuid))
            .order(wiki_page_revisions::revision_number.desc())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(revisions.into_iter().map(WikiPageRevision::from).collect())
    }

    /// Get a specific revision by page ID and revision number
    async fn wiki_page_revision(
        &self,
        ctx: &Context<'_>,
        page_id: ID,
        revision_number: i32,
    ) -> Result<WikiPageRevision> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let page_uuid: Uuid = page_id
            .parse()
            .map_err(|_| TinyBoardsError::NotFound("Invalid page ID".into()))?;

        let revision: DbWikiPageRevision = wiki_page_revisions::table
            .filter(wiki_page_revisions::page_id.eq(page_uuid))
            .filter(wiki_page_revisions::revision_number.eq(revision_number))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Revision not found".into()))?;

        Ok(WikiPageRevision::from(revision))
    }
}
