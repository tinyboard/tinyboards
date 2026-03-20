use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::stream::{
        Stream as DbStream, StreamBoardSubscription as DbStreamBoardSubscription,
        StreamFlairSubscription as DbStreamFlairSubscription, StreamFollower as DbStreamFollower,
    },
    schema::{
        stream_aggregates, stream_board_subscriptions, stream_flair_subscriptions,
        stream_followers,
    },
    utils::{DbPool, get_conn},
};
use uuid::Uuid;

use crate::LoggedInUser;

use super::user::User;

/// GraphQL Stream type
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Stream {
    pub id: ID,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub creator_id: ID,
    pub is_public: bool,
    pub is_discoverable: bool,
    pub sort_type: String,
    pub time_range: Option<String>,
    pub show_nsfw: bool,
    pub max_posts_per_board: Option<i32>,
    pub share_token: Option<String>,
    pub last_viewed_at: Option<String>,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(name = "updatedAt")]
    pub updated_at: String,
    #[graphql(skip)]
    pub(crate) uuid_id: Uuid,
    #[graphql(skip)]
    pub(crate) uuid_creator_id: Uuid,
}

#[ComplexObject]
impl Stream {
    /// Board subscriptions for this stream
    async fn board_subscriptions(&self, ctx: &Context<'_>) -> Result<Vec<StreamBoardSubscription>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let subs: Vec<DbStreamBoardSubscription> = stream_board_subscriptions::table
            .filter(stream_board_subscriptions::stream_id.eq(self.uuid_id))
            .load(conn)
            .await
            .unwrap_or_default();

        Ok(subs.into_iter().map(StreamBoardSubscription::from).collect())
    }

    /// Flair subscriptions for this stream
    async fn flair_subscriptions(&self, ctx: &Context<'_>) -> Result<Vec<StreamFlairSubscription>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let subs: Vec<DbStreamFlairSubscription> = stream_flair_subscriptions::table
            .filter(stream_flair_subscriptions::stream_id.eq(self.uuid_id))
            .load(conn)
            .await
            .unwrap_or_default();

        Ok(subs.into_iter().map(StreamFlairSubscription::from).collect())
    }

    /// Creator user object
    async fn creator(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        use tinyboards_db::schema::users;
        let user = users::table
            .find(self.uuid_creator_id)
            .first::<tinyboards_db::models::user::user::User>(conn)
            .await
            .ok();

        Ok(user.map(|u| User::from(u)))
    }

    /// Follower count from aggregates
    async fn follower_count(&self, ctx: &Context<'_>) -> Result<Option<i32>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let count = stream_aggregates::table
            .filter(stream_aggregates::stream_id.eq(self.uuid_id))
            .select(stream_aggregates::follower_count)
            .first::<i32>(conn)
            .await
            .ok();

        Ok(count)
    }

    /// Board subscription count from aggregates
    async fn board_subscription_count(&self, ctx: &Context<'_>) -> Result<Option<i32>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let count = stream_aggregates::table
            .filter(stream_aggregates::stream_id.eq(self.uuid_id))
            .select(stream_aggregates::board_subscription_count)
            .first::<i32>(conn)
            .await
            .ok();

        Ok(count)
    }

    /// Whether the current user is following this stream
    async fn is_following(&self, ctx: &Context<'_>) -> Result<Option<bool>> {
        let user = ctx.data::<LoggedInUser>()?.inner();
        let Some(u) = user else {
            return Ok(None);
        };

        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let count: i64 = stream_followers::table
            .filter(stream_followers::stream_id.eq(self.uuid_id))
            .filter(stream_followers::user_id.eq(u.id))
            .count()
            .get_result(conn)
            .await
            .unwrap_or(0);

        Ok(Some(count > 0))
    }
}

impl From<DbStream> for Stream {
    fn from(s: DbStream) -> Self {
        let sort_str = match s.sort_type {
            tinyboards_db::enums::DbSortType::Hot => "hot",
            tinyboards_db::enums::DbSortType::New => "new",
            tinyboards_db::enums::DbSortType::Top => "top",
            tinyboards_db::enums::DbSortType::Old => "old",
            tinyboards_db::enums::DbSortType::MostComments => "most_comments",
            tinyboards_db::enums::DbSortType::Controversial => "controversial",
        };
        let uuid_id = s.id;
        let uuid_creator_id = s.creator_id;
        Self {
            id: s.id.to_string().into(),
            name: s.name,
            slug: s.slug,
            description: s.description,
            icon: s.icon,
            color: s.color,
            creator_id: s.creator_id.to_string().into(),
            is_public: s.is_public,
            is_discoverable: s.is_discoverable,
            sort_type: sort_str.to_string(),
            time_range: s.time_range,
            show_nsfw: s.show_nsfw,
            max_posts_per_board: s.max_posts_per_board,
            share_token: s.share_token,
            last_viewed_at: s.last_viewed_at.map(|t| t.to_string()),
            created_at: s.created_at.to_string(),
            updated_at: s.updated_at.to_string(),
            uuid_id,
            uuid_creator_id,
        }
    }
}

/// Flair subscription within a stream
#[derive(SimpleObject, Clone)]
pub struct StreamFlairSubscription {
    pub id: ID,
    pub stream_id: ID,
    pub board_id: ID,
    pub flair_id: i32,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<DbStreamFlairSubscription> for StreamFlairSubscription {
    fn from(sub: DbStreamFlairSubscription) -> Self {
        Self {
            id: sub.id.to_string().into(),
            stream_id: sub.stream_id.to_string().into(),
            board_id: sub.board_id.to_string().into(),
            flair_id: sub.flair_id,
            created_at: sub.created_at.to_string(),
        }
    }
}

/// Board subscription within a stream
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct StreamBoardSubscription {
    pub id: ID,
    pub stream_id: ID,
    pub board_id: ID,
    pub include_all_posts: bool,
    #[graphql(name = "createdAt")]
    pub created_at: String,
    #[graphql(skip)]
    pub(crate) uuid_board_id: Uuid,
}

#[ComplexObject]
impl StreamBoardSubscription {
    /// The board this subscription refers to
    async fn board(&self, ctx: &Context<'_>) -> Result<Option<super::boards::Board>> {
        use crate::{PostgresLoader, newtypes::BoardId};
        use async_graphql::dataloader::DataLoader;
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(BoardId(self.uuid_board_id))
            .await
            .map_err(|e| e.into())
    }
}

impl From<DbStreamBoardSubscription> for StreamBoardSubscription {
    fn from(sub: DbStreamBoardSubscription) -> Self {
        let uuid_board_id = sub.board_id;
        Self {
            id: sub.id.to_string().into(),
            stream_id: sub.stream_id.to_string().into(),
            board_id: sub.board_id.to_string().into(),
            include_all_posts: sub.include_all_posts,
            created_at: sub.created_at.to_string(),
            uuid_board_id,
        }
    }
}

/// Stream follower
#[derive(SimpleObject, Clone)]
pub struct StreamFollower {
    pub id: ID,
    pub user_id: ID,
    pub stream_id: ID,
    pub added_to_navbar: bool,
    pub navbar_position: Option<i32>,
    #[graphql(name = "createdAt")]
    pub created_at: String,
}

impl From<DbStreamFollower> for StreamFollower {
    fn from(f: DbStreamFollower) -> Self {
        Self {
            id: f.id.to_string().into(),
            user_id: f.user_id.to_string().into(),
            stream_id: f.stream_id.to_string().into(),
            added_to_navbar: f.added_to_navbar,
            navbar_position: f.navbar_position,
            created_at: f.created_at.to_string(),
        }
    }
}

/// Navbar settings for a stream
#[derive(SimpleObject, Clone)]
pub struct NavbarSettings {
    pub added_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

/// Input for creating a stream
#[derive(InputObject)]
pub struct CreateStreamInput {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub is_discoverable: Option<bool>,
    pub sort_type: Option<String>,
    pub time_range: Option<String>,
    pub show_nsfw: Option<bool>,
    pub max_posts_per_board: Option<i32>,
}

/// Input for updating a stream
#[derive(InputObject)]
pub struct UpdateStreamInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_public: Option<bool>,
    pub is_discoverable: Option<bool>,
    pub sort_type: Option<String>,
    pub time_range: Option<String>,
    pub show_nsfw: Option<bool>,
    pub max_posts_per_board: Option<i32>,
}

/// Input for adding flair subscriptions
#[derive(InputObject)]
pub struct AddFlairSubscriptionsInput {
    pub stream_id: ID,
    pub board_id: ID,
    pub flair_ids: Vec<i32>,
}

/// Input for adding board subscriptions
#[derive(InputObject)]
pub struct AddBoardSubscriptionsInput {
    pub stream_id: ID,
    pub board_ids: Vec<ID>,
}

/// Sort type for stream discovery
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum StreamSortType {
    #[graphql(name = "new")]
    New,
    #[graphql(name = "popular")]
    Popular,
    #[graphql(name = "trending")]
    Trending,
}
