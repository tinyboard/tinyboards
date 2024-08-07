use std::collections::HashMap;

use crate::{
    structs::{CommentView, LocalUserView},
    DeleteableOrRemoveable,
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    models::{
        board::board_person_bans::BoardPersonBan, board::board_subscriber::BoardSubscriber,
        board::boards::BoardSafe, comment::comment_saved::CommentSaved, comment::comments::Comment,
        person::local_user::*, person::person::*, person::person_blocks::*, post::posts::Post,
    },
    schema::{
        board_mods, board_person_bans, board_subscriber, boards, comment_aggregates, comment_saved,
        comment_votes, comments, person, person_blocks, person_board_blocks, posts,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, fuzzy_search, get_conn, limit_and_offset_unlimited, DbPool},
    CommentSortType, ListingType,
};
use tinyboards_utils::TinyBoardsError;
use typed_builder::TypedBuilder;

type CommentViewTuple = (
    Comment,
    PersonSafe,
    Post,
    BoardSafe,
    CommentAggregates,
    Option<BoardPersonBan>,
    Option<BoardSubscriber>,
    Option<CommentSaved>,
    Option<PersonBlock>,
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

    /// Order comments into a hierarchical tree structure. `top_comment_id` is the id of the comment that counts as top level; if not provided, comments with no `parent_id` will be considered top-level.
    pub fn into_tree(dataset: Vec<Self>, top_comment_id: Option<i32>) -> Vec<Self> {
        // We REALLY don't want to deal with references here! Everything should be OWNED by the object it belongs to.

        // comment id -> list of top level replies
        let mut hash_table: HashMap<i32, Vec<Self>> = HashMap::new();
        let top_comment_id = top_comment_id.unwrap_or(-1);

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
                // determine if the comment is top-level
                if let Some(parent_id) = comment.comment.parent_id {
                    // if the comment is top-level, then it remains in dataset
                    if comment.comment.id == top_comment_id {
                        filtered_dataset.push(comment);
                    } else {
                        // otherwise it will be moved into the hash table, keyed with its parent's id
                        let entry = hash_table.entry(parent_id).or_insert(Vec::new());
                        entry.push(comment);
                    }

                    // continue;
                } else {
                    // if parent id is None, it's definitely top level
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

    pub async fn read(
        pool: &DbPool,
        comment_id: i32,
        my_person_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_id_join = my_person_id.unwrap_or(-1);

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
            .inner_join(person::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_person_bans::table.on(boards::id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriber::table.on(posts::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_saved::table.on(comments::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(comments::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                comments::all_columns,
                PersonSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_person_bans::all_columns.nullable(),
                board_subscriber::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .first::<CommentViewTuple>(conn)
            .await?;

        let my_vote = if my_person_id.is_some() && comment_votes.is_none() {
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
            report_count: None,
        })
    }

    /**
    Returns a comments with a list of its replies up to level 5, ordered into a tree. You can optionally specify `context` to load a specific amount of parent commments as well.
    */
    pub async fn get_comment_with_replies(
        pool: &DbPool,
        comment_id: i32,
        sort: Option<CommentSortType>,
        person: Option<&LocalUserView>,
        context: Option<i32>,
        parent_post_id: Option<i32>,
        is_admin: bool,
        is_mod: bool,
    ) -> Result<CommentQueryResponse, TinyBoardsError> {
        // max allowed value for context is 4
        let context = std::cmp::min(context.unwrap_or(0), 4);
        let person_id = person.map(|u| u.person.id);
        let top_comment = Self::read(pool, comment_id, person_id).await?;

        if let Some(parent_post_id) = parent_post_id {
            if top_comment.comment.post_id != parent_post_id {
                return Err(TinyBoardsError::from_message(
                    400,
                    "That comment doesn't belong to the specified post!",
                ));
            }
        }

        let mut ids = vec![top_comment.comment.id];
        let mut top_comment_id = top_comment.comment.id;
        let parent_comment_id = top_comment.comment.parent_id;
        let mut comments_vec = vec![top_comment];
        let mut total_count: i64 = 1;

        // read parent comments equal to context
        if let Some(mut parent_comment_id) = parent_comment_id {
            for _ in 0..context {
                let parent_comment_view = Self::read(pool, parent_comment_id, person_id).await?;
                top_comment_id = parent_comment_view.comment.id;
                let parent_id = parent_comment_view.comment.parent_id;
                comments_vec.push(parent_comment_view);
                total_count += 1;

                match parent_id {
                    Some(parent_id) => parent_comment_id = parent_id,
                    None => break,
                };
            }
        }

        // load replies, then replies of replies, and so on and so forth
        for _ in 0..(5 - context) {
            let CommentQueryResponse {
                mut comments,
                count,
            } = CommentQuery::builder()
                .pool(pool)
                .parent_ids(Some(&ids))
                .person_id(person_id)
                .show_deleted(Some(true))
                .show_removed(Some(true))
                .sort(sort)
                .build()
                .list()
                .await?;

            total_count += count;
            ids = comments
                .iter()
                .map(|comment_view| comment_view.comment.id)
                .collect();

            // no need for further iterations if there are no more replies present
            if ids.is_empty() {
                break;
            }

            comments_vec.append(&mut comments);
        }

        // hide deleted/removed info
        for cv in comments_vec
            .iter_mut()
            .filter(|cv| cv.comment.is_deleted || cv.comment.is_removed)
        {
            cv.hide_if_removed_or_deleted(person.map(|view| view.person.id), is_admin, is_mod);
        }

        Ok(CommentQueryResponse {
            comments: Self::into_tree(comments_vec, Some(top_comment_id)),
            count: total_count,
        })
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    listing_type: Option<ListingType>,
    user: Option<&'a LocalUser>,
    sort: Option<CommentSortType>,
    board_id: Option<i32>,
    post_id: Option<i32>,
    parent_id: Option<i32>,
    parent_ids: Option<&'a [i32]>,
    creator_id: Option<i32>,
    person_id: Option<i32>,
    search_term: Option<String>,
    saved_only: Option<bool>,
    show_deleted: Option<bool>,
    show_removed: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct CommentQueryResponse {
    pub comments: Vec<CommentView>,
    pub count: i64,
}

impl<'a> CommentQuery<'a> {
    pub async fn list(self) -> Result<CommentQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        use diesel::dsl::*;

        // let person_id_join = self.user.map(|l| l.person_id).unwrap_or(-1);
        let person_id_join = self.person_id.unwrap_or(-1);
        // let local_user_id_join = self.user.map(|l| l.id).unwrap_or(-1);

        let mut query = comments::table
            .inner_join(person::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_person_bans::table.on(boards::id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_subscriber::table.on(posts::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                board_mods::table.on(comments::board_id
                    .eq(board_mods::board_id)
                    .and(board_mods::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_saved::table.on(comments::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(comments::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                comments::all_columns,
                PersonSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_person_bans::all_columns.nullable(),
                board_subscriber::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .into_boxed();

        let mut count_query = comments::table
            .inner_join(person::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_person_bans::table.on(boards::id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(comments::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .left_join(
                board_mods::table.on(comments::board_id
                    .eq(board_mods::board_id)
                    .and(board_mods::person_id.eq(person_id_join))),
            )
            .left_join(
                board_subscriber::table.on(posts::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_saved::table.on(comments::id
                    .eq(comment_saved::comment_id)
                    .and(comment_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(comments::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            .select((
                comments::all_columns,
                PersonSafe::safe_columns_tuple(),
                posts::all_columns,
                BoardSafe::safe_columns_tuple(),
                comment_aggregates::all_columns,
                board_person_bans::all_columns.nullable(),
                board_subscriber::all_columns.nullable(),
                comment_saved::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                comment_votes::score.nullable(),
            ))
            .into_boxed();

        if let Some(creator_id) = self.creator_id {
            query = query.filter(comments::creator_id.eq(creator_id));
            count_query = count_query.filter(comments::creator_id.eq(creator_id));
        };

        if let Some(post_id) = self.post_id {
            query = query.filter(comments::post_id.eq(post_id));
            count_query = count_query.filter(comments::post_id.eq(post_id));
        };

        if let Some(parent_id) = self.parent_id {
            query = query.filter(comments::parent_id.eq(parent_id));
            count_query = count_query.filter(comments::parent_id.eq(parent_id));
        };

        if let Some(parent_ids) = self.parent_ids {
            query = query.filter(comments::parent_id.eq_any(parent_ids));
            count_query = count_query.filter(comments::parent_id.eq_any(parent_ids));
        }

        if let Some(search_term) = self.search_term {
            query = query.filter(comments::body.ilike(fuzzy_search(&search_term)));
            count_query = count_query.filter(comments::body.ilike(fuzzy_search(&search_term)));
        };

        if let Some(listing_type) = self.listing_type {
            match listing_type {
                ListingType::Subscribed => {
                    query = query.filter(board_subscriber::person_id.is_not_null());
                    count_query = count_query.filter(board_subscriber::person_id.is_not_null());
                }
                ListingType::All => {
                    query = query.filter(
                        boards::is_hidden
                            .eq(false)
                            .or(board_subscriber::person_id.eq(person_id_join)),
                    );
                    count_query = count_query.filter(
                        boards::is_hidden
                            .eq(false)
                            .or(board_subscriber::person_id.eq(person_id_join)),
                    );
                }
                ListingType::Local => {
                    query = query.filter(boards::local.eq(true));
                    count_query = count_query.filter(boards::local.eq(true));
                }
                ListingType::Moderated => {
                    query = query.filter(board_mods::person_id.is_not_null());
                    count_query = count_query.filter(board_mods::person_id.is_not_null());
                }
            }
        };

        if let Some(board_id) = self.board_id {
            query = query.filter(posts::board_id.eq(board_id));
            count_query = count_query.filter(posts::board_id.eq(board_id));
        }

        if self.saved_only.unwrap_or(false) {
            query = query.filter(comment_saved::id.is_not_null());
            count_query = count_query.filter(comment_saved::id.is_not_null());
        }

        if !self.show_deleted.unwrap_or(false) {
            query = query.filter(comments::is_deleted.eq(false));

            count_query = count_query.filter(comments::is_deleted.eq(false));
        }

        if !self.show_removed.unwrap_or(false) {
            query = query.filter(comments::is_removed.eq(false));

            count_query = count_query.filter(comments::is_removed.eq(false));
        }

        if self.person_id.is_some() {
            query = query.filter(person_board_blocks::person_id.is_null());
            query = query.filter(person_blocks::person_id.is_null());

            count_query = count_query.filter(person_board_blocks::person_id.is_null());
            count_query = count_query.filter(person_blocks::person_id.is_null());
        }

        if !self.user.map(|u| u.show_bots).unwrap_or(true) {
            query = query.filter(person::bot_account.eq(false));
            count_query = count_query.filter(person::bot_account.eq(false));
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
            .load::<CommentViewTuple>(conn)
            .await?;

        let comments = CommentView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;

        Ok(CommentQueryResponse { comments, count })
    }
}

impl DeleteableOrRemoveable for CommentView {
    fn hide_if_removed_or_deleted(&mut self, user_id: Option<i32>, is_admin: bool, is_mod: bool) {
        // if the user is admin or mod, nothing is being removed
        /*if let Some(user_view) = user_view {
        if user_view.local_user.has_permission(AdminPerms::Content) {
            return;
        }
        }*/

        if is_admin {
            return;
        }

        let blank_out_comment = {
            if self.comment.is_removed {
                match user_id {
                    Some(user_id) => {
                        // the user can read the comment if they are its creator or is a board mod (deleted is blank for everyone)
                        !(is_mod || user_id == self.comment.creator_id)
                    }
                    None => true,
                }
            } else {
                self.comment.is_deleted
            }
        };

        if blank_out_comment {
            let obscure_text: String = {
                if self.comment.is_deleted {
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
            if self.post.is_removed {
                match user_id {
                    Some(user_id) => !(user_id == self.post.creator_id || is_mod),
                    None => true,
                }
            } else {
                self.post.is_deleted
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

            //self.post.title = obscure_text.clone();
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
                report_count: None,
            })
            .collect::<Vec<Self>>()
    }
}
