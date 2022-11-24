use std::collections::HashMap;

use crate::{
    structs::{CommentView, UserView},
    DeleteableOrRemoveable,
};
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    models::{
        board::board::BoardSafe,
        board::board_subscriber::BoardSubscriber,
        board::board_user_ban::BoardUserBan,
        comment::comment::Comment,
        comment::comment_saved::CommentSaved,
        post::post::Post,
        user::user::UserSafe,
        user::user_block::UserBlock,
    },
    schema::{
        board, board_block, board_subscriber, board_user_ban, comment, comment_aggregates,
        comment_saved, comment_vote, post, user_, user_block,
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
            comment_vote,
        ) = comment::table
            .find(comment_id)
            .inner_join(user_::table)
            .inner_join(post::table)
            .inner_join(board::table.on(post::board_id.eq(board::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_user_ban::table.on(board::id
                    .eq(board_user_ban::board_id)
                    .and(board_user_ban::user_id.eq(comment::creator_id))
                    .and(
                        board_user_ban::expires
                            .is_null()
                            .or(board_user_ban::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriber::table.on(post::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_saved::table.on(comment::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::user_id.eq(user_id_join))),
            )
            .left_join(
                user_block::table.on(comment::creator_id
                    .eq(user_block::target_id)
                    .and(user_block::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_vote::table.on(comment::id
                    .eq(comment_vote::comment_id)
                    .and(comment_vote::user_id.eq(user_id_join))),
            )
            .select((
                comment::all_columns,
                UserSafe::safe_columns_tuple(),
                post::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_user_ban::all_columns.nullable(),
                board_subscriber::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                user_block::all_columns.nullable(),
                comment_vote::score.nullable(),
            ))
            .first::<CommentViewTuple>(conn)?;

        let my_vote = if my_user_id.is_some() && comment_vote.is_none() {
            Some(0)
        } else {
            comment_vote
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
    user: Option<&'a UserSafe>,
    search_term: Option<String>,
    saved_only: Option<bool>,
    show_deleted_and_removed: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

impl<'a> CommentQuery<'a> {
    pub fn list(self) -> Result<Vec<CommentView>, Error> {
        use diesel::dsl::*;

        let user_id_join = self.user.map(|l| l.id).unwrap_or(-1);

        let mut query = comment::table
            .inner_join(user_::table)
            .inner_join(post::table)
            .inner_join(board::table.on(post::board_id.eq(board::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_user_ban::table.on(board::id
                    .eq(board_user_ban::board_id)
                    .and(board_user_ban::user_id.eq(comment::creator_id))
                    .and(
                        board_user_ban::expires
                            .is_null()
                            .or(board_user_ban::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriber::table.on(post::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_saved::table.on(comment::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::user_id.eq(user_id_join))),
            )
            .left_join(
                user_block::table.on(comment::creator_id
                    .eq(user_block::target_id)
                    .and(user_block::user_id.eq(user_id_join))),
            )
            .left_join(
                board_block::table.on(board::id
                    .eq(board_block::board_id)
                    .and(board_block::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_vote::table.on(comment::id
                    .eq(comment_vote::comment_id)
                    .and(comment_vote::user_id.eq(user_id_join))),
            )
            .select((
                comment::all_columns,
                UserSafe::safe_columns_tuple(),
                post::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_user_ban::all_columns.nullable(),
                board_subscriber::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                user_block::all_columns.nullable(),
                comment_vote::score.nullable(),
            ))
            .into_boxed();

        if let Some(creator_id) = self.creator_id {
            query = query.filter(comment::creator_id.eq(creator_id));
        };

        if let Some(post_id) = self.post_id {
            query = query.filter(comment::post_id.eq(post_id));
        };

        if let Some(parent_id) = self.parent_id {
            query = query.filter(comment::parent_id.eq(parent_id));
        };

        if let Some(search_term) = self.search_term {
            query = query.filter(comment::body.ilike(fuzzy_search(&search_term)));
        };

        if let Some(listing_type) = self.listing_type {
            match listing_type {
                ListingType::Subscribed => {
                    query = query.filter(board_subscriber::user_id.is_not_null())
                }
                ListingType::All => {
                    query = query.filter(
                        board::hidden
                            .eq(false)
                            .or(board_subscriber::user_id.eq(user_id_join)),
                    )
                }
            }
        };

        if let Some(board_id) = self.board_id {
            query = query.filter(post::board_id.eq(board_id));
        }

        if self.saved_only.unwrap_or(false) {
            query = query.filter(comment_saved::id.is_not_null());
        }

        if !self.show_deleted_and_removed.unwrap_or(true) {
            query = query.filter(comment::removed.eq(false));
            query = query.filter(comment::deleted.eq(false));
        }

        if self.user.is_some() {
            query = query.filter(board_block::user_id.is_null());
            query = query.filter(user_block::user_id.is_null());
        }

        let (limit, offset) = limit_and_offset_unlimited(self.page, self.limit);

        // comment ordering logic here

        query = match self.sort.unwrap_or(CommentSortType::Hot) {
            CommentSortType::Hot => query
                .then_order_by(
                    hot_rank(comment_aggregates::score, comment_aggregates::published).desc(),
                )
                .then_order_by(comment_aggregates::published.desc()),
            CommentSortType::New => query.then_order_by(comment::published.desc()),
            CommentSortType::Old => query.then_order_by(comment::published.asc()),
            CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
        };

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<CommentViewTuple>(self.conn)?;

        Ok(CommentView::from_tuple_to_vec(res))
    }
}

impl DeleteableOrRemoveable for CommentView {
    fn hide_if_removed_or_deleted(&mut self, user_view: Option<&UserView>) {
        // if the user is admin, nothing is being removed
        if let Some(user_view) = user_view {
            if user_view.user.admin {
                return;
            }
        }

        let blank_out_comment = {
            if self.comment.removed || self.comment.deleted {
                match user_view {
                    Some(user_view) => {
                        // the user can read the comment if they are its creator (deleted is blank for everyone)
                        !(self.comment.removed && user_view.user.id == self.comment.creator_id)
                    }
                    None => true,
                }
            } else {
                false
            }
        };

        if blank_out_comment {
            let obscure_text: String = {
                if self.comment.deleted {
                    "[ deleted ]"
                } else {
                    "[ removed ]"
                }
            }
            .into();

            self.comment.body = obscure_text.clone();
            self.comment.body_html = obscure_text;
            self.comment.creator_id = -1;
            self.creator = None;
        }

        let blank_out_post = {
            if self.post.deleted || self.post.removed {
                match user_view {
                    Some(user_view) => {
                        !(self.post.removed && user_view.user.id == self.post.creator_id)
                    }
                    None => true,
                }
            } else {
                false
            }
        };

        // also blank out post
        if blank_out_post {
            let obscure_text: String = {
                if self.post.deleted {
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
