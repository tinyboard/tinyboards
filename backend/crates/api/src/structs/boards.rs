use async_graphql::*;
use tinyboards_db::models::board::boards::Board as DbBoard;
use tinyboards_db::models::board::board_mods::BoardModerator as DbBoardMod;
use tinyboards_db::models::aggregates::BoardAggregates as DbBoardAggregates;

/// GraphQL Board type.
#[derive(SimpleObject, Clone)]
pub struct Board {
    pub id: ID,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub sidebar: Option<String>,
    #[graphql(name = "sidebarHTML")]
    pub sidebar_html: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub hover_color: String,
    #[graphql(name = "isNSFW")]
    pub is_nsfw: bool,
    pub is_hidden: bool,
    pub is_removed: bool,
    pub is_banned: bool,
    pub is_posting_restricted_to_mods: bool,
    pub exclude_from_all: bool,
    pub public_ban_reason: Option<String>,
    pub wiki_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    // Aggregate counts
    pub subscribers: i64,
    pub posts: i64,
    pub comments: i64,
    pub users_active_day: i64,
    pub users_active_week: i64,
    pub users_active_month: i64,
    pub users_active_half_year: i64,
    pub is_subscribed: bool,
    pub section_config: i32,
    pub custom_css: Option<String>,
}

impl Board {
    pub fn from_db(board: DbBoard, agg: Option<DbBoardAggregates>) -> Self {
        Self::from_db_with_sub(board, agg, false)
    }

    pub fn from_db_with_sub(board: DbBoard, agg: Option<DbBoardAggregates>, is_subscribed: bool) -> Self {
        let (subscribers, posts, comments, users_active_day, users_active_week, users_active_month, users_active_half_year) = match agg {
            Some(a) => (a.subscribers, a.posts, a.comments, a.users_active_day, a.users_active_week, a.users_active_month, a.users_active_half_year),
            None => (0, 0, 0, 0, 0, 0, 0),
        };
        let section_config = board.section_config;
        Self {
            id: ID(board.id.to_string()),
            name: board.name,
            title: board.title,
            description: board.description,
            sidebar: board.sidebar,
            sidebar_html: board.sidebar_html,
            icon: board.icon,
            banner: board.banner,
            primary_color: board.primary_color,
            secondary_color: board.secondary_color,
            hover_color: board.hover_color,
            is_nsfw: board.is_nsfw,
            is_hidden: board.is_hidden,
            is_removed: board.is_removed,
            is_banned: board.is_banned,
            is_posting_restricted_to_mods: board.is_posting_restricted_to_mods,
            exclude_from_all: board.exclude_from_all,
            public_ban_reason: board.public_ban_reason,
            wiki_enabled: board.wiki_enabled,
            created_at: board.created_at.to_rfc3339(),
            updated_at: board.updated_at.to_rfc3339(),
            subscribers,
            posts,
            comments,
            users_active_day,
            users_active_week,
            users_active_month,
            users_active_half_year,
            is_subscribed,
            section_config,
            custom_css: board.custom_css,
        }
    }
}

/// Board moderator info for GraphQL.
#[derive(SimpleObject, Clone)]
pub struct BoardMod {
    pub id: ID,
    pub board_id: ID,
    pub user_id: ID,
    pub permissions: i32,
    pub rank: i32,
    pub is_invite_accepted: bool,
    pub created_at: String,
}

impl From<DbBoardMod> for BoardMod {
    fn from(m: DbBoardMod) -> Self {
        Self {
            id: ID(m.id.to_string()),
            board_id: ID(m.board_id.to_string()),
            user_id: ID(m.user_id.to_string()),
            permissions: m.permissions,
            rank: m.rank,
            is_invite_accepted: m.is_invite_accepted,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}
