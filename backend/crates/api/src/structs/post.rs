use crate::PostgresLoader;
use async_graphql::*;
use async_graphql::dataloader::DataLoader;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::PostAggregates as DbPostAggregates,
        post::posts::Post as DbPost,
        reaction::Reaction as DbReaction,
    },
    schema::{post_flairs, reactions, reaction_aggregates},
    utils::{DbPool, get_conn},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{
    newtypes::{BoardId, ModPermsForBoardId, UserId, SavedForPostId, VoteForPostId},
    Censorable, LoggedInUser,
};

use super::{boards::Board, user::User};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Post {
    pub id: ID,
    pub title: String,
    pub post_type: String,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub body: String,
    #[graphql(name = "bodyHTML")]
    pub body_html: String,
    pub creator_id: ID,
    pub board_id: ID,
    pub is_removed: bool,
    pub is_locked: bool,
    pub created_at: String,
    pub is_deleted: bool,
    #[graphql(name = "isNSFW")]
    pub is_nsfw: bool,
    pub updated_at: String,
    pub image: Option<String>,
    pub is_featured_board: bool,
    pub is_featured_local: bool,
    pub alt_text: Option<String>,
    pub embed_title: Option<String>,
    pub embed_description: Option<String>,
    pub embed_video_url: Option<String>,
    pub source_url: Option<String>,
    pub last_crawl_date: Option<String>,
    pub slug: String,
    pub is_thread: bool,
    pub approval_status: String,
    // Internal UUID fields for dataloaders
    #[graphql(skip)]
    pub(crate) uuid_id: Uuid,
    #[graphql(skip)]
    pub(crate) uuid_creator_id: Uuid,
    #[graphql(skip)]
    pub(crate) uuid_board_id: Uuid,
    #[graphql(skip)]
    counts: DbPostAggregates,
}

#[ComplexObject]
impl Post {
    pub async fn comment_count(&self) -> i64 {
        self.counts.comments
    }

    pub async fn score(&self) -> i64 {
        self.counts.score
    }

    pub async fn upvotes(&self) -> i64 {
        self.counts.upvotes
    }

    pub async fn downvotes(&self) -> i64 {
        self.counts.downvotes
    }

    pub async fn newest_comment_time(&self) -> String {
        self.counts.newest_comment_time.to_rfc3339()
    }

    pub async fn hot_rank(&self) -> i32 {
        self.counts.hot_rank
    }

    pub async fn hot_rank_active(&self) -> i32 {
        self.counts.hot_rank_active
    }

    pub async fn controversy_rank(&self) -> f64 {
        self.counts.controversy_rank
    }

    /// Get the canonical URL for this post
    pub async fn url_path(&self, ctx: &Context<'_>) -> Result<String> {
        use crate::utils::url_builder::{get_boards_enabled, UrlBuilder};
        let pool = ctx.data::<DbPool>()?;

        let boards_enabled = get_boards_enabled(pool).await?;
        let url_builder = UrlBuilder::new(boards_enabled);

        let board_slug = if boards_enabled {
            self.board(ctx).await?.map(|b| b.name.clone())
        } else {
            None
        };

        let url = if self.is_thread {
            url_builder.build_thread_url(self.uuid_id, &self.slug, board_slug.as_deref())
        } else {
            url_builder.build_feed_url(self.uuid_id, &self.slug, board_slug.as_deref())
        };
        Ok(url)
    }

    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(UserId(self.uuid_creator_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn board(&self, ctx: &Context<'_>) -> Result<Option<Board>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(BoardId(self.uuid_board_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn my_vote(&self, ctx: &Context<'_>) -> Result<i32> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(VoteForPostId(self.uuid_id))
            .await
            .map(|v| v.unwrap_or(0))
            .map_err(|e| e.into())
    }

    pub async fn my_mod_permissions(&self, ctx: &Context<'_>) -> Result<i32> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(ModPermsForBoardId(self.uuid_board_id))
            .await
            .map(|v| v.unwrap_or(0))
            .map_err(|e| e.into())
    }

    pub async fn is_saved(&self, ctx: &Context<'_>) -> Result<bool> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(SavedForPostId(self.uuid_id))
            .await
            .map(|v| v.unwrap_or(false))
            .map_err(|e| e.into())
    }

    /// Get aggregated reaction counts for this post
    pub async fn reaction_counts(&self, ctx: &Context<'_>) -> Result<Vec<super::reaction::ReactionAggregate>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let aggregates: Vec<tinyboards_db::models::aggregates::ReactionAggregates> =
            reaction_aggregates::table
                .filter(reaction_aggregates::post_id.eq(self.uuid_id))
                .load(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(aggregates.into_iter().map(super::reaction::ReactionAggregate::from).collect())
    }

    /// Get the current user's reaction to this post (if any)
    pub async fn my_reaction(&self, ctx: &Context<'_>) -> Result<Option<super::reaction::Reaction>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data::<LoggedInUser>()?.inner();

        if let Some(u) = user {
            let conn = &mut get_conn(pool).await?;
            let reaction: Option<DbReaction> = reactions::table
                .filter(reactions::post_id.eq(self.uuid_id))
                .filter(reactions::user_id.eq(u.id))
                .first(conn)
                .await
                .optional()
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            Ok(reaction.map(super::reaction::Reaction::from))
        } else {
            Ok(None)
        }
    }

    /// Get flairs assigned to this post
    pub async fn flairs(&self, ctx: &Context<'_>) -> Result<Vec<super::flair::PostFlair>> {
        use tinyboards_db::schema::post_flairs;
        use tinyboards_db::models::flair::PostFlair as PostFlairDb;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let post_flairs_list: Vec<PostFlairDb> = post_flairs::table
            .filter(post_flairs::post_id.eq(self.uuid_id))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(post_flairs_list.into_iter().map(super::flair::PostFlair::from).collect())
    }
}

impl Censorable for Post {
    fn censor(&mut self, my_user_id: uuid::Uuid, is_admin: bool, is_mod: bool) {
        if !(self.is_removed || self.is_deleted) {
            return;
        }

        if is_admin {
            return;
        }

        // you can see your own removed content
        if self.is_removed && (is_mod || self.uuid_creator_id == my_user_id) {
            return;
        }

        let obscure_text = if self.is_deleted {
            "[ deleted by creator ]"
        } else {
            "[ removed by mod ]"
        }
        .to_string();

        // more strict censoring for deleted posts
        if self.is_deleted {
            self.title = obscure_text.clone();
            self.creator_id = ID("".to_string());
        }

        self.body = obscure_text.clone();
        self.body_html = obscure_text;
        self.url = None;
    }
}

impl From<(DbPost, DbPostAggregates)> for Post {
    fn from((post, counts): (DbPost, DbPostAggregates)) -> Self {
        let is_deleted = post.deleted_at.is_some();
        Self {
            id: ID(post.id.to_string()),
            title: post.title,
            post_type: format!("{:?}", post.post_type).to_lowercase(),
            url: post.url.clone(),
            thumbnail_url: post.thumbnail_url.clone(),
            body: post.body.clone(),
            body_html: post.body_html.clone(),
            creator_id: ID(post.creator_id.to_string()),
            board_id: ID(post.board_id.to_string()),
            is_removed: post.is_removed,
            is_locked: post.is_locked,
            created_at: post.created_at.to_rfc3339(),
            is_deleted,
            is_nsfw: post.is_nsfw,
            updated_at: post.updated_at.to_rfc3339(),
            image: post.image.clone(),
            is_featured_board: post.is_featured_board,
            is_featured_local: post.is_featured_local,
            alt_text: post.alt_text.clone(),
            embed_title: post.embed_title.clone(),
            embed_description: post.embed_description.clone(),
            embed_video_url: post.embed_video_url.clone(),
            source_url: post.source_url.clone(),
            last_crawl_date: post.last_crawl_date.map(|d| d.to_rfc3339()),
            slug: post.slug.clone(),
            is_thread: post.is_thread,
            approval_status: format!("{:?}", post.approval_status).to_lowercase(),
            uuid_id: post.id,
            uuid_creator_id: post.creator_id,
            uuid_board_id: post.board_id,
            counts,
        }
    }
}
