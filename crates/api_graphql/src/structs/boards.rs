use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::{
    aggregates::structs::BoardAggregates as DbBoardAggregates,
    models::{
        board::{board_mods::BoardModerator as DbBoardMod, boards::Board as DbBoard},
        person::local_user::AdminPerms,
        post::posts::Post as DbPost,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    newtypes::{ModPermsForBoardId, SubscribedTypeForBoardId},
    ListingType, LoggedInUser, PostgresLoader, SortType, SubscribedType,
};

use super::{board_mods::BoardMod, post::Post};

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
    #[graphql(name = "isNSFW")]
    is_nsfw: bool,
    actor_id: String,
    subscribers_url: String,
    inbox_url: String,
    shared_inbox_url: Option<String>,
    moderators_url: Option<String>,
    featured_url: Option<String>,
    icon: Option<String>,
    banner: Option<String>,
    #[graphql(name = "isBanned")]
    is_removed: bool,
    ban_reason: Option<String>,
    primary_color: String,
    secondary_color: String,
    hover_color: String,
    sidebar: Option<String>,
    #[graphql(name = "sidebarHTML")]
    sidebar_html: Option<String>,
    // `counts` is not queryable, fields will be made available through resolvers
    #[graphql(skip)]
    counts: DbBoardAggregates,
    // this value is only accessible to admins thru a resolver
    #[graphql(skip)]
    hidden_: bool,
}

// resolvers for BoardAggregates fields
#[ComplexObject]
impl Board {
    // we do a little trolling
    pub async fn is_hidden(&self, ctx: &Context<'_>) -> bool {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        match v_opt {
            Some(v) => v.local_user.has_permission(AdminPerms::Boards) && self.hidden_,
            None => false,
        }
    }

    pub async fn subscribers(&self) -> i64 {
        self.counts.subscribers
    }

    pub async fn post_count(&self) -> i64 {
        self.counts.posts
    }

    pub async fn comment_count(&self) -> i64 {
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

    pub async fn my_mod_permissions(&self, ctx: &Context<'_>) -> Result<i32> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(ModPermsForBoardId(self.id))
            .await
            .map(|v| v.unwrap_or(0))
            .map_err(|e| e.into())
    }

    pub async fn subscribed_type(&self, ctx: &Context<'_>) -> Result<SubscribedType> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(SubscribedTypeForBoardId(self.id))
            .await
            .map(|s| s.unwrap_or(SubscribedType::NotSubscribed))
            .map_err(|e| e.into())
    }

    pub async fn moderators(&self, ctx: &Context<'_>) -> Result<Vec<BoardMod>> {
        let pool = ctx.data_unchecked::<DbPool>();

        DbBoardMod::for_board(pool, self.id)
            .await
            .map(|res| {
                res.into_iter()
                    .map(BoardMod::from)
                    .collect::<Vec<BoardMod>>()
            })
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load board mods.").into()
            })
    }

    pub async fn posts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many posts to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Sorting type.")] sort: Option<SortType>,
        #[graphql(desc = "If specified, only posts from the given user will be loaded.")]
        person_id: Option<i32>,
        #[graphql(desc = "Page.")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let sort = sort.unwrap_or(SortType::NewComments);
        let listing_type = ListingType::All;
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };
        // If the board is banned (or deleted), only admins can view its posts
        let can_view_posts = if self.is_removed || self.is_deleted {
            match v_opt {
                Some(v) => v.local_user.has_permission(AdminPerms::Boards),
                None => false,
            }
        } else {
            true
        };

        let posts = DbPost::load_with_counts(
            pool,
            person_id_join,
            Some(limit),
            page,
            false,
            false,
            can_view_posts,
            false,
            Some(self.id),
            person_id,
            sort.into(),
            listing_type.into(),
        )
        .await?;

        Ok(posts.into_iter().map(Post::from).collect::<Vec<Post>>())
    }
}

impl From<(DbBoard, DbBoardAggregates)> for Board {
    fn from((board, counts): (DbBoard, DbBoardAggregates)) -> Self {
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
            hidden_: board.is_hidden,
            counts,
        }
    }
}

impl From<(&DbBoard, &DbBoardAggregates)> for Board {
    fn from((ref_board, ref_counts): (&DbBoard, &DbBoardAggregates)) -> Self {
        Self::from((ref_board.clone(), ref_counts.clone()))
    }
}
