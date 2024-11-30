use async_graphql::*;
use dataloader::DataLoader;
use tinyboards_db::{
    aggregates::structs::PostAggregates as DbPostAggregates,
    models::{
        board::board_mods::{BoardModerator as DbBoardMod, ModPerms},
        comment::comments::Comment as DbComment,
        person::local_user::AdminPerms,
        post::posts::Post as DbPost,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{
    newtypes::{BoardId, ModPermsForBoardId, PersonId, SavedForPostId, VoteForPostId},
    Censorable, CommentSortType, ListingType, LoggedInUser, PostgresLoader,
};

use super::{boards::Board, comment::Comment, person::Person};

#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Post {
    id: i32,
    title: String,
    type_: String,
    url: Option<String>,
    body: String,
    #[graphql(name = "bodyHTML")]
    body_html: String,
    creator_id: i32,
    board_id: i32,
    is_removed: bool,
    is_locked: bool,
    creation_date: String,
    is_deleted: bool,
    #[graphql(name = "isNSFW")]
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

    pub async fn downvotes(&self) -> i64 {
        self.counts.downvotes
    }

    pub async fn newest_comment_time(&self) -> String {
        self.counts.newest_comment_time.to_string()
    }

    pub async fn creator(&self, ctx: &Context<'_>) -> Result<Option<Person>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(PersonId(self.creator_id))
            .await
            .map_err(|e| e.into())
    }

    pub async fn board(&self, ctx: &Context<'_>) -> Result<Option<Board>> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();
        loader
            .load_one(BoardId(self.board_id))
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

    pub async fn my_mod_permissions(&self, ctx: &Context<'_>) -> Result<i32> {
        let loader = ctx.data_unchecked::<DataLoader<PostgresLoader>>();

        loader
            .load_one(ModPermsForBoardId(self.board_id))
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

    pub async fn comments(
        &self,
        ctx: &Context<'_>,
        sort: Option<CommentSortType>,
        listing_type: Option<ListingType>,
        page: Option<i64>,
        limit: Option<i64>,
        //no_tree: Option<bool>,
        search: Option<String>,
        top_comment_id: Option<i32>,
        context: Option<u16>,
        include_deleted: Option<bool>,
        include_removed: Option<bool>,
        max_depth: Option<i32>,
    ) -> Result<Vec<Comment>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };
        let is_admin = match v_opt {
            Some(v) => v.local_user.has_permission(AdminPerms::Content),
            None => false,
        };
        let is_mod = match v_opt {
            Some(v) => {
                let mod_rel =
                    DbBoardMod::get_by_person_id_for_board(pool, v.person.id, self.board_id, true)
                        .await;
                match mod_rel {
                    Ok(m) => m.has_permission(ModPerms::Content),
                    Err(_) => false,
                }
            }
            None => false,
        };

        // We do not nest comments if there is a search, or the user explicitly doesn't want it
        //  let no_tree = search.is_some() || no_tree.unwrap_or(false);
        // Default sort type
        let sort = sort.unwrap_or(CommentSortType::Hot);
        // Setting listing type is only allowed if we don't nest comments: for nesting, we always need all comments
        let listing_type = listing_type.unwrap_or(ListingType::All);
        // we only load removed comments if we are nesting comments, or the user can view removed comments (is mod or admin)
        // let include_removed = !no_tree || is_admin || is_mod;
        // same here, except only admins can see deleted comments
        // let include_deleted = !no_tree || is_admin;

        let include_removed = include_removed.unwrap_or(true);
        let include_deleted = include_deleted.unwrap_or(true);

        let max_depth = max_depth.unwrap_or(6);

        // `tree_top_comment_id` is the id of the top comment.
        // This may differ from the provided `top_comment_id` if context > 0, because then we're requesting the parent comments of the top comment
        let (_, mut comments) = if let Some(top_comment_id) = top_comment_id {
            let (tree_top_comment_id, db_comments_with_counts) =
                DbComment::get_with_replies_and_counts(
                    pool,
                    top_comment_id,
                    context,
                    sort.into(),
                    person_id_join,
                    Some(self.id),
                    Some(max_depth),
                )
                .await?;

            let comments = db_comments_with_counts
                .into_iter()
                .map(Comment::from)
                .collect::<Vec<Comment>>();

            (Some(tree_top_comment_id), comments)
        } else {
            let comments = DbComment::load_with_counts(
                pool,
                person_id_join,
                sort.into(),
                listing_type.into(),
                page,
                limit,
                None,
                Some(self.id),
                None,
                false,
                search,
                // inclulde deleted and removed:
                include_deleted,
                include_removed,
                true,
                None,
                Some(max_depth),
            )
            .await?
            .into_iter()
            .map(Comment::from)
            .collect::<Vec<Comment>>();

            (None, comments)
        };

        // if we are nesting comments and the user isn't an admin, censor removed and deleted comments
        if !is_admin {
            for comment in comments
                .iter_mut()
                .filter(|comment| comment.is_removed || comment.is_deleted)
            {
                comment.censor(person_id_join, is_admin, is_mod);
            }
        }

        // nest/tree comments
        /*if !no_tree {
            comments = Comment::tree(comments, tree_top_comment_id);
        }*/

        Ok(comments)
    }
}

impl Censorable for Post {
    fn censor(&mut self, my_person_id: i32, is_admin: bool, is_mod: bool) {
        // do nothing
        if !(self.is_removed || self.is_deleted) {
            return;
        }

        // do nothing if admin either
        if is_admin {
            return;
        }

        // you can see your own removed content
        if self.is_removed && (is_mod || self.creator_id == my_person_id) {
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
            self.creator_id = -1;
        }

        self.body = obscure_text.clone();
        self.body_html = obscure_text;

        self.url = None;
        self.type_ = "text".to_string();
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
