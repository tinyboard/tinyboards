use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::{
    aggregates::structs::PostAggregates as DbPostAggregates,
    models::{comment::comments::Comment as DbComment, post::posts::Post as DbPost},
    newtypes::UserId,
    utils::DbPool,
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::TinyBoardsError;

use crate::{
    newtypes::{BoardIdForPost, SavedForPostId, VoteForPostId},
    LoggedInUser, PostgresLoader,
};

use super::{boards::Board, person::Person};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct Post {
    id: i32,
    title: String,
    type_: String,
    url: Option<String>,
    body: String,
    body_html: String,
    creator_id: i32,
    board_id: i32,
    is_removed: bool,
    is_locked: bool,
    creation_date: String,
    is_deleted: bool,
    is_nsfw: bool,
    updated: Option<String>,
    image: Option<String>,
    local: bool,
    featured_board: bool,
    featured_local: bool,
    title_chunk: String,
    #[graphql(skip)]
    counts: DbPostAggregates,
    /*creator: Option<Person>,
    is_creator_banned_from_board: bool,
    is_saved: bool,
    my_vote: Option<i16>,
    mod_permissions: Option<i32>,*/
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

    pub async fn newest_comment_time(&self) -> String {
        self.counts.newest_comment_time.to_string()
    }

    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<Person>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(UserId(self.creator_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn board(&self, ctx: &Context<'_>) -> Result<Option<Board>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(BoardIdForPost(self.board_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn my_vote(&self, ctx: &Context<'_>) -> Result<i16> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(VoteForPostId(self.id))
            .await
            .map(|v| v.unwrap_or(0))
            .map_err(|e| e.into())
    }

    pub async fn is_saved(&self, ctx: &Context<'_>) -> Result<bool> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(SavedForPostId(self.id))
            .await
            .map(|v| v.unwrap_or(false))
            .map_err(|e| e.into())
    }

    pub async fn participants(&self, ctx: &Context<'_>) -> Result<Vec<Person>> {
        let pool = ctx.data_unchecked::<DbPool>();

        let resp = DbComment::load_participants_for_post(pool, self.id)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load participants for post.")
            })?;

        Ok(resp.into_iter().map(Person::from).collect::<Vec<Person>>())
    }
}

impl From<(DbPost, DbPostAggregates)> for Post {
    #[allow(unused)]
    fn from((post, counts): (DbPost, DbPostAggregates)) -> Self {
        Self {
            id: post.id,
            title: post.title,
            type_: post.type_,
            url: post.url.map(|url| url.as_str().into()),
            body: post.body,
            body_html: post.body_html,
            creator_id: post.creator_id,
            board_id: post.board_id,
            is_removed: post.is_removed,
            is_locked: post.is_locked,
            creation_date: post.creation_date.to_string(),
            is_deleted: post.is_deleted,
            is_nsfw: post.is_nsfw,
            updated: post.updated.map(|t| t.to_string()),
            image: post.image.map(|i| i.as_str().into()),
            local: post.local,
            featured_board: post.featured_board,
            featured_local: post.featured_local,
            title_chunk: post.title_chunk,
            counts,
            /*creator: creator.map(|c| Person::from((c, creator_counts.unwrap()))),
            is_creator_banned_from_board: creator_banned_from_board,
            is_saved: saved,
            my_vote,
            mod_permissions,*/
        }
    }
}

/*impl From<PostView> for Post {
    #[allow(unused)]
    fn from(
        PostView {
            post,
            creator,
            creator_counts,
            board,
            creator_banned_from_board,
            counts,
            subscribed,
            saved,
            read,
            creator_blocked,
            my_vote,
            report_count,
            mod_permissions,
        }: PostView,
    ) -> Self {
        Self {
            id: post.id,
            title: post.title,
            type_: post.type_,
            url: post.url.map(|url| url.as_str().into()),
            body: post.body,
            body_html: post.body_html,
            creator_id: post.creator_id,
            board_id: post.board_id,
            is_removed: post.is_removed,
            is_locked: post.is_locked,
            creation_date: post.creation_date.to_string(),
            is_deleted: post.is_deleted,
            is_nsfw: post.is_nsfw,
            updated: post.updated.map(|t| t.to_string()),
            image: post.image.map(|i| i.as_str().into()),
            local: post.local,
            featured_board: post.featured_board,
            featured_local: post.featured_local,
            title_chunk: post.title_chunk,
            counts,
            creator: creator.map(|c| Person::from((c, creator_counts.unwrap()))),
            is_creator_banned_from_board: creator_banned_from_board,
            is_saved: saved,
            my_vote,
            mod_permissions,
        }
    }
}*/
