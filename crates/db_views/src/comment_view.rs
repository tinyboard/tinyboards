use std::collections::HashMap;

use crate::{structs::CommentView, DeleteableOrRemoveable};
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    models::{
        board::board_subscriptions::BoardSubscriber,
        board::board_user_bans::BoardUserBan,
        board::boards::BoardSafe,
        comment::comments::Comment,
        comment::user_comment_save::CommentSaved,
        post::posts::Post,
        user::user_blocks::UserBlock,
        user::users::{User, UserSafe},
    },
    schema::{
        board_subscriptions, board_user_bans, boards, comment_aggregates, comment_votes, comments,
        posts, user_blocks, user_board_blocks, user_comment_save, users,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, fuzzy_search, limit_and_offset_unlimited},
    CommentSortType, ListingType,
};
use typed_builder::TypedBuilder;

type CommentViewTuple = (
    Comment,
    UserSafe,
    Post,
    BoardSafe,
    CommentAggregates,
    Option<BoardUserBan>,
    Option<BoardSubscriber>,
    Option<CommentSaved>,
    Option<UserBlock>,
    Option<i16>,
);

impl CommentView {
    /// this function takes a CommentView, and adds its array of replies based on the hash table provided
    fn tree_wrap(self, hash_table: &mut HashMap<i32, Vec<Self>>) -> Self {
        Self {
            replies: {
                let mut replies: Vec<Self> = Vec::new();

                // if this comment has children stored in the hash_table, claim them!
                if let Some(children) = hash_table.remove(&self.comment.id) {
                    // and for each, repeat this.
                    for child in children.into_iter() {
                        replies.push(Self::tree_wrap(child, hash_table));
                    }
                }

                replies
            },
            ..self
        }
    }

    /// Order comments into a hierarchical tree structure.
    pub fn into_tree(dataset: Vec<Self>) -> Vec<Self> {
        // We REALLY don't want to deal with references here! Everything should be OWNED by the object it belongs to.

        // comment id -> list of top level replies
        let mut hash_table: HashMap<i32, Vec<Self>> = HashMap::new();

        /*for comment in dataset.iter() {
            if let Some(parent_id) = comment.comment.parent_id {
                let entry = hash_table.entry(parent_id).or_insert(Vec::new());
                entry.push(comment);
            }
        }*/
        let dataset = {
            // we only want top-level comments to remain in dataset, therefore...
            let mut filtered_dataset = Vec::new();

            for comment in dataset.into_iter() {
                // if the comment is not top-level, then...
                if let Some(parent_id) = comment.comment.parent_id {
                    // it will be moved into the hash table, keyed with its parent's id
                    let entry = hash_table.entry(parent_id).or_insert(Vec::new());
                    entry.push(comment);

                    continue;
                } else {
                    // otherwise it remains in dataset
                    filtered_dataset.push(comment);
                }
            }

            filtered_dataset
        };

        let mut tree: Vec<Self> = Vec::new();

        // call tree_wrap in each item to populate their replies array
        for comment in dataset.into_iter() {
            tree.push(comment.tree_wrap(&mut hash_table));
        }

        tree
    }

    pub fn read(
        conn: &mut PgConnection,
        comment_id: i32,
        my_user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_id_join = my_user_id.unwrap_or(-1);

        let (
            comment,
            creator,
            post,
            board,
            counts,
            creator_banned_from_board,
            subscriber,
            saved,
            creator_blocked,
            comment_votes,
        ) = comments::table
            .find(comment_id)
            .inner_join(users::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(comments::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_comment_save::table.on(comments::id
                    .eq(user_comment_save::comment_id)
                    .and(user_comment_save::user_id.eq(user_id_join))),
            )
            .left_join(
                user_blocks::table.on(comments::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::user_id.eq(user_id_join))),
            )
            .select((
                comments::all_columns,
                UserSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_user_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                user_comment_save::all_columns.nullable(),
                user_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .first::<CommentViewTuple>(conn)?;

        let my_vote = if my_user_id.is_some() && comment_votes.is_none() {
            Some(0)
        } else {
            comment_votes
        };

        Ok(CommentView {
            comment,
            creator: Some(creator),
            post,
            board,
            counts,
            creator_banned_from_board: creator_banned_from_board.is_some(),
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
            saved: saved.is_some(),
            creator_blocked: creator_blocked.is_some(),
            my_vote,
            replies: Vec::with_capacity(0),
        })
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    listing_type: Option<ListingType>,
    sort: Option<CommentSortType>,
    board_id: Option<i32>,
    post_id: Option<i32>,
    parent_id: Option<i32>,
    creator_id: Option<i32>,
    user_id: Option<i32>,
    search_term: Option<String>,
    saved_only: Option<bool>,
    show_deleted_and_removed: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct CommentQueryResponse {
    pub comments: Vec<CommentView>,
    pub count: i64,
}

impl<'a> CommentQuery<'a> {
    pub fn list(self) -> Result<CommentQueryResponse, Error> {
        use diesel::dsl::*;

        let user_id_join = self.user_id.unwrap_or(-1);

        let mut query = comments::table
            .inner_join(users::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(comments::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_comment_save::table.on(comments::id
                    .eq(user_comment_save::comment_id)
                    .and(user_comment_save::user_id.eq(user_id_join))),
            )
            .left_join(
                user_blocks::table.on(comments::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                user_board_blocks::table.on(boards::id
                    .eq(user_board_blocks::board_id)
                    .and(user_board_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::user_id.eq(user_id_join))),
            )
            .select((
                comments::all_columns,
                UserSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_user_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                user_comment_save::all_columns.nullable(),
                user_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .into_boxed();

        let count_query = comments::table
            .inner_join(users::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_user_bans::table.on(boards::id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(comments::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_comment_save::table.on(comments::id
                    .eq(user_comment_save::comment_id)
                    .and(user_comment_save::user_id.eq(user_id_join))),
            )
            .left_join(
                user_blocks::table.on(comments::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                user_board_blocks::table.on(boards::id
                    .eq(user_board_blocks::board_id)
                    .and(user_board_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::user_id.eq(user_id_join))),
            )
            .select((
                comments::all_columns,
                UserSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_user_bans::all_columns.nullable(),
                board_subscriptions::all_columns.nullable(),
                user_comment_save::all_columns.nullable(),
                user_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .into_boxed();

        if let Some(creator_id) = self.creator_id {
            query = query.filter(comments::creator_id.eq(creator_id));
        };

        if let Some(post_id) = self.post_id {
            query = query.filter(comments::post_id.eq(post_id));
        };

        if let Some(parent_id) = self.parent_id {
            query = query.filter(comments::parent_id.eq(parent_id));
        };

        if let Some(search_term) = self.search_term {
            query = query.filter(comments::body.ilike(fuzzy_search(&search_term)));
        };

        if let Some(listing_type) = self.listing_type {
            match listing_type {
                ListingType::Subscribed => {
                    query = query.filter(board_subscriptions::user_id.is_not_null())
                }
                ListingType::All => {
                    query = query.filter(
                        boards::is_hidden
                            .eq(false)
                            .or(board_subscriptions::user_id.eq(user_id_join)),
                    )
                }
            }
        };

        if let Some(board_id) = self.board_id {
            query = query.filter(posts::board_id.eq(board_id));
        }

        if self.saved_only.unwrap_or(false) {
            query = query.filter(user_comment_save::id.is_not_null());
        }

        if !self.show_deleted_and_removed.unwrap_or(false) {
            query = query.filter(comments::is_removed.eq(false));
            query = query.filter(comments::is_deleted.eq(false));
        }

        if self.user_id.is_some() {
            query = query.filter(user_board_blocks::user_id.is_null());
            query = query.filter(user_blocks::user_id.is_null());
        }

        let (limit, offset) = limit_and_offset_unlimited(self.page, self.limit);

        // comment ordering logic here

        query = match self.sort.unwrap_or(CommentSortType::Hot) {
            CommentSortType::Hot => query
                .then_order_by(
                    hot_rank(comment_aggregates::score, comment_aggregates::creation_date).desc(),
                )
                .then_order_by(comment_aggregates::creation_date.desc()),
            CommentSortType::New => query.then_order_by(comments::creation_date.desc()),
            CommentSortType::Old => query.then_order_by(comments::creation_date.asc()),
            CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
        };

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<CommentViewTuple>(self.conn)?;

        let comments = CommentView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(self.conn)?;

        Ok(CommentQueryResponse { comments, count })
    }
}

impl DeleteableOrRemoveable for CommentView {
    fn hide_if_removed_or_deleted(&mut self, user: Option<&User>) {
        // if the user is admin, nothing is being removed
        if let Some(user) = user {
            if user.is_admin {
                return;
            }
        }

        let blank_out_comment = {
            if self.comment.is_removed || self.comment.is_deleted {
                match user {
                    Some(user) => {
                        // the user can read the comment if they are its creator (deleted is blank for everyone)
                        !(self.comment.is_removed && user.id == self.comment.creator_id)
                    }
                    None => true,
                }
            } else {
                false
            }
        };

        if blank_out_comment {
            let obscure_text: String = {
                if self.comment.is_deleted {
                    "[ retracted ]"
                } else {
                    "[ purged ]"
                }
            }
            .into();

            self.comment.body = obscure_text.clone();
            self.comment.body_html = obscure_text;
            self.comment.creator_id = -1;
            self.creator = None;
        }

        let blank_out_post = {
            if self.post.is_deleted || self.post.is_removed {
                match user {
                    Some(user) => !(self.post.is_removed && user.id == self.post.creator_id),
                    None => true,
                }
            } else {
                false
            }
        };

        // also blank out post
        if blank_out_post {
            let obscure_text: String = {
                if self.post.is_deleted {
                    "[ deleted ]"
                } else {
                    "[ removed ]"
                }
            }
            .into();

            self.post.title = obscure_text.clone();
            self.post.body = obscure_text.clone();
            self.post.body_html = obscure_text;
            self.post.url = None;
            self.post.creator_id = -1;
        }
    }
}

impl ViewToVec for CommentView {
    type DbTuple = CommentViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                comment: a.0,
                creator: Some(a.1),
                post: a.2,
                board: a.3,
                counts: a.4,
                creator_banned_from_board: a.5.is_some(),
                subscribed: BoardSubscriber::to_subscribed_type(&a.6),
                saved: a.7.is_some(),
                creator_blocked: a.8.is_some(),
                my_vote: a.9,
                replies: Vec::with_capacity(0),
            })
            .collect::<Vec<Self>>()
    }
}
