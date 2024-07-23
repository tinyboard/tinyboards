use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::BoardAggregates as DbBoardAggregates,
    models::board::boards::{Board as DbBoard, BoardSafe as DbBoardSafe},
};

/// GraphQL representation of Board.
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Board {
    pub id: i32,
    name: String,
    title: String,
    description: Option<String>,
    creation_date: String,
    updated: Option<String>,
    is_deleted: bool,
    is_nsfw: bool,
    is_hidden: bool,
    actor_id: String,
    subscribers_url: String,
    inbox_url: String,
    shared_inbox_url: Option<String>,
    moderators_url: Option<String>,
    featured_url: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    is_removed: bool,
    ban_reason: Option<String>,
    primary_color: String,
    secondary_color: String,
    hover_color: String,
    sidebar: Option<String>,
    sidebar_html: Option<String>,
    // `counts` is not queryable, fields will be made available through resolvers
    #[graphql(skip)]
    counts: DbBoardAggregates,
}


// resolvers for BoardAggregates fields
#[ComplexObject]
impl Board {
    pub async fn subscribers(&self) -> i64 {
        self.counts.subscribers
    }

    pub async fn posts(&self) -> i64 {
        self.counts.posts
    }

    pub async fn comments(&self) -> i64 {
        self.counts.comments
    }

    pub async fn users_active_day(&self) -> i64 {
        self.counts.users_active_day
    }

    pub async fn users_active_week(&self) -> i64 {
        self.counts.users_active_week
    }

    pub async fn users_active_month(&self) -> i64 {
        self.counts.users_active_month
    }

    pub async fn users_active_half_year(&self) -> i64 {
        self.counts.users_active_half_year
    }
}

impl From<(DbBoard, DbBoardAggregates)> for Board {
    fn from((board, counts): (DbBoardSafe, DbBoardAggregates)) -> Self {
        Self {
            id: board.id,
            name: board.name,
            title: board.title,
            icon: board.icon.map(|a| a.as_str().into()),
            banner: board.banner.map(|a| a.as_str().into()),
            description: board.description,
            creation_date: board.creation_date.to_string(),
            updated: board.updated.map(|t| t.to_string()),
            is_deleted: board.is_deleted,
            is_removed: board.is_removed,
            is_nsfw: board.is_nsfw,
            is_hidden: board.is_hidden,
            actor_id: board.actor_id.to_string(),
            subscribers_url: board.subscribers_url.to_string(),
            inbox_url: board.inbox_url.to_string(),
            shared_inbox_url: board.shared_inbox_url.map(|a| a.as_str().into()),
            moderators_url: board.moderators_url.map(|a| a.as_str().into()),
            featured_url: board.featured_url.map(|a| a.as_str().into()),
            ban_reason: board.ban_reason,
            primary_color: board.primary_color,
            secondary_color: board.secondary_color,
            hover_color: board.hover_color,
            sidebar: board.sidebar,
            sidebar_html: board.sidebar_html,
            counts,
        }
    }
}

impl From<(&DbBoard, &DbBoardAggregates)> for Board {
    fn from((ref_board, ref_counts): (&DbBoard, &DbBoardAggregates)) -> Self {
        Self::from((ref_board.clone(), ref_counts.clone()))
    }
}