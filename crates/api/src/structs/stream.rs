use async_graphql::*;

// Re-export from tinyboards_db when implemented
// For now, we define the expected structures that will be provided by the DB layer

/// GraphQL Stream type
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Stream {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub creator_id: i32,
    pub is_public: bool,
    pub share_token: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub last_viewed_at: Option<String>,
    pub max_posts_per_board: Option<i32>,
    #[graphql(skip)]
    pub(crate) aggregates: StreamAggregatesData,
}

#[ComplexObject]
impl Stream {
    /// Get the stream creator
    async fn creator(&self, ctx: &Context<'_>) -> Result<Option<super::user::User>> {
        use crate::newtypes::UserId;
        use crate::PostgresLoader;
        use dataloader::DataLoader;

        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(UserId(self.creator_id))
            .await
            .map_err(|e| e.into())
    }

    /// Get flair subscriptions for this stream
    async fn flair_subscriptions(&self, ctx: &Context<'_>) -> Result<Vec<StreamFlairSubscription>> {
        use tinyboards_db::models::stream::stream_flair_subscription::StreamFlairSubscription as DbStreamFlairSubscription;
        use tinyboards_utils::TinyBoardsError;

        let pool = ctx.data::<crate::DbPool>()?;

        let subscriptions = DbStreamFlairSubscription::list_for_stream(pool, self.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load flair subscriptions"))?;

        Ok(subscriptions.into_iter().map(StreamFlairSubscription::from).collect())
    }

    /// Get board subscriptions for this stream
    async fn board_subscriptions(&self, ctx: &Context<'_>) -> Result<Vec<StreamBoardSubscription>> {
        use tinyboards_db::models::stream::stream_board_subscription::StreamBoardSubscription as DbStreamBoardSubscription;
        use tinyboards_utils::TinyBoardsError;

        let pool = ctx.data::<crate::DbPool>()?;

        let subscriptions = DbStreamBoardSubscription::list_for_stream(pool, self.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load board subscriptions"))?;

        Ok(subscriptions.into_iter().map(StreamBoardSubscription::from).collect())
    }

    /// Get followers of this stream
    async fn followers(&self, ctx: &Context<'_>, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<StreamFollower>> {
        use tinyboards_db::models::stream::stream_follower::StreamFollower as DbStreamFollower;
        use tinyboards_utils::TinyBoardsError;

        let pool = ctx.data::<crate::DbPool>()?;

        let followers = DbStreamFollower::list_for_stream(pool, self.id, limit, offset)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load followers"))?;

        Ok(followers.into_iter().map(StreamFollower::from).collect())
    }

    /// Get aggregated statistics
    async fn follower_count(&self) -> i64 {
        self.aggregates.follower_count
    }

    async fn total_subscriptions(&self) -> i32 {
        self.aggregates.total_subscriptions
    }

    async fn flair_subscription_count(&self) -> i32 {
        self.aggregates.flair_subscription_count
    }

    async fn board_subscription_count(&self) -> i32 {
        self.aggregates.board_subscription_count
    }

    /// Check if current user is following this stream
    async fn is_following(&self, ctx: &Context<'_>) -> Result<bool> {
        use crate::LoggedInUser;
        use tinyboards_db::models::stream::stream_follower::StreamFollower as DbStreamFollower;

        let pool = ctx.data::<crate::DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        if let Some(u) = user {
            Ok(DbStreamFollower::is_following(pool, u.id, self.id).await.unwrap_or(false))
        } else {
            Ok(false)
        }
    }

    /// Check if current user has stream pinned to navbar
    async fn navbar_settings(&self, ctx: &Context<'_>) -> Result<Option<NavbarSettings>> {
        use crate::LoggedInUser;
        use tinyboards_db::models::stream::stream_follower::StreamFollower as DbStreamFollower;

        let pool = ctx.data::<crate::DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        if let Some(u) = user {
            if let Ok(follower) = DbStreamFollower::get(pool, u.id, self.id).await {
                if follower.added_to_navbar {
                    return Ok(Some(NavbarSettings {
                        add_to_navbar: true,
                        navbar_position: follower.navbar_position,
                    }));
                }
            }
        }

        Ok(None)
    }
}

/// Internal struct for aggregate data (not exposed directly to GraphQL)
#[derive(Clone, Default)]
pub struct StreamAggregatesData {
    pub follower_count: i64,
    pub total_subscriptions: i32,
    pub flair_subscription_count: i32,
    pub board_subscription_count: i32,
}

/// Flair subscription within a stream
#[derive(SimpleObject, Clone)]
pub struct StreamFlairSubscription {
    pub id: i32,
    pub stream_id: i32,
    pub flair_id: i32,
    pub board_id: i32,
    pub added_at: String,
}

/// Board subscription within a stream (subscribe to ALL content from a board)
#[derive(SimpleObject, Clone)]
pub struct StreamBoardSubscription {
    pub id: i32,
    pub stream_id: i32,
    pub board_id: i32,
    pub added_at: String,
}

/// Stream follower
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct StreamFollower {
    pub user_id: i32,
    pub stream_id: i32,
    pub followed_at: String,
    pub add_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

#[ComplexObject]
impl StreamFollower {
    async fn user(&self, ctx: &Context<'_>) -> Result<Option<super::user::User>> {
        use crate::newtypes::UserId;
        use crate::PostgresLoader;
        use dataloader::DataLoader;

        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(UserId(self.user_id))
            .await
            .map_err(|e| e.into())
    }
}

/// Navbar settings for a stream
#[derive(SimpleObject, Clone)]
pub struct NavbarSettings {
    pub add_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

/// Input for creating a stream
#[derive(InputObject)]
pub struct CreateStreamInput {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub max_posts_per_board: Option<i32>,
}

/// Input for updating a stream
#[derive(InputObject)]
pub struct UpdateStreamInput {
    pub stream_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub max_posts_per_board: Option<i32>,
}

/// Input for adding flair subscriptions
#[derive(InputObject)]
pub struct AddFlairSubscriptionsInput {
    pub stream_id: i32,
    pub flair_ids: Vec<i32>,
}

/// Input for removing a flair subscription
#[derive(InputObject)]
pub struct RemoveFlairSubscriptionInput {
    pub stream_id: i32,
    pub flair_id: i32,
}

/// Input for adding board subscriptions
#[derive(InputObject)]
pub struct AddBoardSubscriptionsInput {
    pub stream_id: i32,
    pub board_ids: Vec<i32>,
}

/// Input for removing a board subscription
#[derive(InputObject)]
pub struct RemoveBoardSubscriptionInput {
    pub stream_id: i32,
    pub board_id: i32,
}

/// Input for following a stream
#[derive(InputObject)]
pub struct FollowStreamInput {
    pub stream_id: i32,
    pub add_to_navbar: Option<bool>,
    pub navbar_position: Option<i32>,
}

/// Input for updating navbar settings
#[derive(InputObject)]
pub struct UpdateStreamNavbarInput {
    pub stream_id: i32,
    pub add_to_navbar: bool,
    pub navbar_position: Option<i32>,
}

/// Sort type for streams
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum StreamSortType {
    #[graphql(name = "new")]
    New,
    #[graphql(name = "old")]
    Old,
    #[graphql(name = "popular")]
    Popular,
    #[graphql(name = "trending")]
    Trending,
}

/// Time range for filtering
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum TimeRange {
    #[graphql(name = "day")]
    Day,
    #[graphql(name = "week")]
    Week,
    #[graphql(name = "month")]
    Month,
    #[graphql(name = "year")]
    Year,
    #[graphql(name = "all")]
    All,
}

// Conversion implementations from database types
// These will be used once the database layer is implemented

impl From<(
    tinyboards_db::models::stream::stream::Stream,
    tinyboards_db::aggregates::structs::StreamAggregates,
)> for Stream {
    fn from(
        (stream, aggregates): (
            tinyboards_db::models::stream::stream::Stream,
            tinyboards_db::aggregates::structs::StreamAggregates,
        ),
    ) -> Self {
        Self {
            id: stream.id,
            name: stream.name,
            slug: stream.slug,
            description: stream.description,
            creator_id: stream.creator_id,
            is_public: stream.is_public,
            share_token: stream.share_token,
            created_at: stream.created_at.to_string(),
            updated_at: stream.updated_at.map(|t| t.to_string()),
            last_viewed_at: stream.last_viewed_at.map(|t| t.to_string()),
            max_posts_per_board: stream.max_posts_per_board,
            aggregates: StreamAggregatesData {
                follower_count: aggregates.follower_count as i64,
                total_subscriptions: aggregates.total_subscription_count,
                flair_subscription_count: aggregates.flair_subscription_count,
                board_subscription_count: aggregates.board_subscription_count,
            },
        }
    }
}

impl From<tinyboards_db::models::stream::stream_flair_subscription::StreamFlairSubscription>
    for StreamFlairSubscription
{
    fn from(sub: tinyboards_db::models::stream::stream_flair_subscription::StreamFlairSubscription) -> Self {
        Self {
            id: sub.id,
            stream_id: sub.stream_id,
            flair_id: sub.flair_id,
            board_id: sub.board_id,
            added_at: sub.creation_date.to_string(),
        }
    }
}

impl From<tinyboards_db::models::stream::stream_board_subscription::StreamBoardSubscription>
    for StreamBoardSubscription
{
    fn from(sub: tinyboards_db::models::stream::stream_board_subscription::StreamBoardSubscription) -> Self {
        Self {
            id: sub.id,
            stream_id: sub.stream_id,
            board_id: sub.board_id,
            added_at: sub.creation_date.to_string(),
        }
    }
}

impl From<tinyboards_db::models::stream::stream_follower::StreamFollower> for StreamFollower {
    fn from(follower: tinyboards_db::models::stream::stream_follower::StreamFollower) -> Self {
        Self {
            user_id: follower.user_id,
            stream_id: follower.stream_id,
            followed_at: follower.followed_at.to_string(),
            add_to_navbar: follower.added_to_navbar,
            navbar_position: follower.navbar_position,
        }
    }
}
