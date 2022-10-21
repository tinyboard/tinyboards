use crate::structs::CommentView;
use diesel::{dsl::*, result::Error, *};
use porpl_db::{
    aggregates::structs::CommentAggregates,
    schema::{
        comment,
        comment_aggregates,
        comment_like,
        comment_saved,
        board,
        board_block,
        board_subscriber,
        board_user_ban,
        user_,
        user_block,
        post,
    },
    models::{
        comment::comment::Comment,
        comment::comment_saved::CommentSaved,
        board::board::BoardSafe,
        board::board_subscriber::BoardSubscriber,
        board::board_user_ban::BoardUserBan,
        user::user::{UserSafe, User},
        user::user_block::UserBlock,
        post::post::Post,
    },
    traits::{ToSafe, ViewToVec},
    utils::{
        functions::hot_rank,
        fuzzy_search,
        limit_and_offset_unlimited,
    },
    CommentSortType,
    ListingType,
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
    pub fn read(
        conn: &mut PgConnection,
        comment_id: i32,
        user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_id_join = user_id.unwrap_or(-1);

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
            comment_like,
        ) = comment::table
            .find(comment_id)
            .inner_join(user_::table)
            .inner_join(post::table)
            .inner_join(board::table.on(post::board_id.eq(board::id)))
            .inner_join(comment_aggregates::table)
            .left_join(
                board_user_ban::table.on(
                    board::id
                        .eq(board_user_ban::board_id)
                        .and(board_user_ban::user_id.eq(comment::creator_id))
                        .and(
                            board_user_ban::expires
                                .is_null()
                                .or(board_user_ban::expires.gt(now)),
                    ),
                ),
            )
            .left_join(
                board_subscriber::table.on(
                    post::board_id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                comment_saved::table.on(
                    comment::id
                        .eq(comment_saved::comment_id)
                        .and(comment_saved::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                user_block::table.on(
                    comment::creator_id
                        .eq(user_block::target_id)
                        .and(user_block::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                comment_like::table.on(
                    comment::id
                        .eq(comment_like::comment_id)
                        .and(comment_like::user_id.eq(user_id_join)),
                ),
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
                comment_like::score.nullable(),
            ))
            .first::<CommentViewTuple>(conn)?;

            let my_vote = if user_id.is_some() && comment_like.is_none() {
                Some(0)
            } else {
                comment_like
            };

            Ok(CommentView {
                comment,
                creator,
                post,
                board,
                counts,
                creator_banned_from_board: creator_banned_from_board.is_some(),
                subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
                saved: saved.is_some(),
                creator_blocked: creator_blocked.is_some(),
                my_vote,
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
    user: Option<&'a User>,
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
                board_user_ban::table.on(
                    board::id
                        .eq(board_user_ban::board_id)
                        .and(board_user_ban::user_id.eq(comment::creator_id))
                        .and(
                            board_user_ban::expires
                                .is_null()
                                .or(board_user_ban::expires.gt(now)),
                    ),
                ),
            )
            .left_join(
                board_subscriber::table.on(
                    post::board_id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                comment_saved::table.on(
                    comment::id
                        .eq(comment_saved::comment_id)
                        .and(comment_saved::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                user_block::table.on(
                    comment::creator_id
                        .eq(user_block::target_id)
                        .and(user_block::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                board_block::table.on(
                    board::id
                        .eq(board_block::board_id)
                        .and(board_block::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                comment_like::table.on(
                    comment::id
                        .eq(comment_like::comment_id)
                        .and(comment_like::user_id.eq(user_id_join)),
                ),
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
                comment_like::score.nullable(),
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
                },
                ListingType::All => {
                    query = query.filter(
                        board::hidden
                            .eq(false)
                            .or(board_subscriber::user_id.eq(user_id_join))
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

        let (limit, offset) = 
            limit_and_offset_unlimited(self.page, self.limit);


        // comment ordering logic here

        query = match self.sort.unwrap_or(CommentSortType::Hot) {
            CommentSortType::Hot => query
                .then_order_by(hot_rank(comment_aggregates::score, comment_aggregates::published).desc())
                .then_order_by(comment_aggregates::published.desc()),
            CommentSortType::New => query
                .then_order_by(comment::published.desc()),
            CommentSortType::Old => query
                .then_order_by(comment::published.asc()),
            CommentSortType::Top => query
                .order_by(comment_aggregates::score.desc()),
        };

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<CommentViewTuple>(self.conn)?;
        
        Ok(CommentView::from_tuple_to_vec(res))
    }
}

impl ViewToVec for CommentView {
    type DbTuple = CommentViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                comment: a.0,
                creator: a.1,
                post: a.2,
                board: a.3,
                counts: a.4,
                creator_banned_from_board: a.5.is_some(),
                subscribed: BoardSubscriber::to_subscribed_type(&a.6),
                saved: a.7.is_some(),
                creator_blocked: a.8.is_some(),
                my_vote: a.9,
            })
            .collect::<Vec<Self>>()
    }
}