use async_graphql::*;
use tinyboards_db::models::wiki::{
    WikiApprovedContributor, WikiPage as DbWikiPage, WikiPageRevision as DbWikiPageRevision,
};

#[derive(SimpleObject, Clone)]
pub struct WikiPage {
    pub id: ID,
    pub board_id: ID,
    pub slug: String,
    pub title: String,
    pub body: String,
    #[graphql(name = "bodyHTML")]
    pub body_html: String,
    pub creator_id: ID,
    pub last_edited_by: Option<ID>,
    pub view_permission: String,
    pub edit_permission: String,
    pub is_locked: bool,
    pub display_order: Option<i32>,
    pub parent_id: Option<ID>,
    pub is_deleted: bool,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
}

impl From<DbWikiPage> for WikiPage {
    fn from(page: DbWikiPage) -> Self {
        let view_perm = match page.view_permission {
            tinyboards_db::enums::DbWikiPermission::Public => "public",
            tinyboards_db::enums::DbWikiPermission::Members => "members",
            tinyboards_db::enums::DbWikiPermission::ModsOnly => "mods_only",
            tinyboards_db::enums::DbWikiPermission::Locked => "locked",
        };
        let edit_perm = match page.edit_permission {
            tinyboards_db::enums::DbWikiPermission::Public => "public",
            tinyboards_db::enums::DbWikiPermission::Members => "members",
            tinyboards_db::enums::DbWikiPermission::ModsOnly => "mods_only",
            tinyboards_db::enums::DbWikiPermission::Locked => "locked",
        };
        WikiPage {
            id: page.id.to_string().into(),
            board_id: page.board_id.to_string().into(),
            slug: page.slug,
            title: page.title,
            body: page.body,
            body_html: page.body_html,
            creator_id: page.creator_id.to_string().into(),
            last_edited_by: page.last_edited_by.map(|id| id.to_string().into()),
            view_permission: view_perm.to_string(),
            edit_permission: edit_perm.to_string(),
            is_locked: page.is_locked,
            display_order: page.display_order,
            parent_id: page.parent_id.map(|id| id.to_string().into()),
            is_deleted: page.deleted_at.is_some(),
            created_at: page.created_at.to_string(),
            updated_at: page.updated_at.to_string(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct WikiPageRevision {
    pub id: ID,
    pub page_id: ID,
    pub revision_number: i32,
    pub editor_id: ID,
    pub edit_summary: Option<String>,
    pub body: String,
    #[graphql(name = "bodyHTML")]
    pub body_html: String,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<DbWikiPageRevision> for WikiPageRevision {
    fn from(rev: DbWikiPageRevision) -> Self {
        WikiPageRevision {
            id: rev.id.to_string().into(),
            page_id: rev.page_id.to_string().into(),
            revision_number: rev.revision_number,
            editor_id: rev.editor_id.to_string().into(),
            edit_summary: rev.edit_summary,
            body: rev.body,
            body_html: rev.body_html,
            created_at: rev.created_at.to_string(),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct WikiApprovedContributorView {
    pub id: ID,
    pub board_id: ID,
    pub user_id: ID,
    pub added_by: ID,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<WikiApprovedContributor> for WikiApprovedContributorView {
    fn from(c: WikiApprovedContributor) -> Self {
        WikiApprovedContributorView {
            id: c.id.to_string().into(),
            board_id: c.board_id.to_string().into(),
            user_id: c.user_id.to_string().into(),
            added_by: c.added_by.to_string().into(),
            created_at: c.created_at.to_string(),
        }
    }
}

// Input types for wiki mutations

#[derive(InputObject)]
pub struct CreateWikiPageInput {
    pub slug: String,
    pub title: String,
    pub body: String,
    pub view_permission: Option<String>,
    pub edit_permission: Option<String>,
    pub display_order: Option<i32>,
    pub parent_id: Option<ID>,
}

#[derive(InputObject)]
pub struct EditWikiPageInput {
    pub title: Option<String>,
    pub body: Option<String>,
    pub edit_summary: Option<String>,
    pub view_permission: Option<String>,
    pub edit_permission: Option<String>,
    pub is_locked: Option<bool>,
    pub display_order: Option<i32>,
}
