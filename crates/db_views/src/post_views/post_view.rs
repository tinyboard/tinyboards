use crate::local_structs::PostView;
use diesel::{dsl::*, pg::Pg, result::Error, *};
use porpl_db::{
    aggregates::structs::PostAggregates,
    schema::{
        board::{self},
        board_block,
        board_subscriber,
        board_user_ban,
        user_,
        user_block,
        post,
        post_aggregates,
        post_like,
        post_read,
        post_saved,
    },
    models::{
        board::board::BoardSafe,
        board::board_subscriber::BoardSubscriber,
        board::board_user_ban::BoardUserBan,
        user::{user::UserSafe, user_block::UserBlock},
        post::post::Post,
        post::post_read::PostRead,
        post::post_saved::PostSaved,
        user::user::User,
    },
    traits::{ToSafe, ViewToVec},
    ListingType,
    SortType,
    utils::{limit_and_offset, fuzzy_search, functions::hot_rank}, porpl_types::UserId,
};
use tracing::debug;
use typed_builder::TypedBuilder;

type PostViewTuple = (
    Post,
    UserSafe,
    BoardSafe,
    Option<BoardUserBan>,
    PostAggregates,
    Option<BoardSubscriber>,
    Option<PostSaved>,
    Option<PostRead>,
    Option<UserBlock>,
    Option<i16>,
);

sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);

impl PostView {
    pub fn read(
        conn: &mut PgConnection,
        post_id: i32,
        my_user_id: Option<i32>,
    ) -> Result<Self, Error> {
        let user_id_join = my_user_id.unwrap_or(-1);
        let (
            post,
            creator,
            board,
            creator_banned_from_board,
            counts,
            subscriber,
            saved,
            read,
            creator_blocked,
            post_like,
        ) = post::table
            .find(post_id)
            .inner_join(user_::table)
            .inner_join(board::table)
            .left_join(
                board_user_ban::table.on(
                    post::board_id
                        .eq(board_user_ban::board_id)
                        .and(board_user_ban::user_id.eq(post::creator_id))
                        .and(
                            board_user_ban::expires
                                .is_null()
                                .or(board_user_ban::expires.gt(now))
                        ),
                    ),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriber::table.on(
                    post::board_id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                post_saved::table.on(
                    post::id
                        .eq(post_saved::post_id)
                        .and(post_saved::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                post_read::table.on(
                    post::id
                        .eq(post_read::post_id)
                        .and(post_read::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                user_block::table.on(
                    post::creator_id
                        .eq(user_block::target_id)
                        .and(user_block::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                post_like::table.on(
                    post::id
                        .eq(post_like::post_id)
                        .and(post_like::user_id.eq(user_id_join)),
                ),
            )
            .select((
                post::all_columns,
                UserSafe::safe_columns_tuple(),
                BoardSafe::safe_columns_tuple(),
                board_user_ban::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                post_saved::all_columns.nullable(),
                post_read::all_columns.nullable(),
                user_block::all_columns.nullable(),
                post_like::score.nullable(),
            ))
            .first::<PostViewTuple>(conn)?;

            let my_vote = if my_user_id.is_some() && post_like.is_none() {
                Some(0)
            } else {
                post_like
            };

            Ok(PostView {
                post,
                creator,
                board,
                creator_banned_from_board: creator_banned_from_board.is_some(),
                counts,
                subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
                saved: saved.is_some(),
                read: read.is_some(),
                creator_blocked: creator_blocked.is_some(),
                my_vote,
            })
    }
}


#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostQuery<'a> {
    #[builder(!default)]
    conn: &'a mut PgConnection,
    listing_type: Option<ListingType>,
    sort: Option<SortType>,
    creator_id: Option<i32>,
    board_id: Option<i32>,
    user: Option<&'a User>,
    search_term: Option<String>,
    url_search: Option<String>,
    saved_only: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

impl<'a> PostQuery<'a> {
    pub fn list(self) -> Result<Vec<PostView>, Error> {
        use diesel::dsl::*;

        let user_id_join = self.user.map(|l| l.id).unwrap_or(-1);

        println!("inside le function, user_id_join = {}", &user_id_join);

        let mut query = post::table
            .inner_join(user_::table)
            .inner_join(board::table)
            .left_join(
                board_user_ban::table.on(
                    post::board_id
                        .eq(board_user_ban::board_id)
                        .and(board_user_ban::user_id.eq(post::creator_id))
                        .and(board_user_ban::expires.is_null().or(board_user_ban::expires.gt(now))
                    ),
                ),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriber::table.on(
                    post::board_id
                        .eq(board_subscriber::board_id)
                        .and(board_subscriber::user_id.eq(user_id_join))
                )
            )
            .left_join(
                post_saved::table.on(
                    post::id
                        .eq(post_saved::post_id)
                        .and(post_saved::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                post_read::table.on(
                    post::id
                        .eq(post_read::post_id)
                        .and(post_read::user_id.eq(user_id_join)),
                ),
            )
            .left_join(
                user_block::table.on(
                    post::creator_id
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
                post_like::table.on(
                    post::id
                        .eq(post_like::post_id)
                        .and(post_like::user_id.eq(user_id_join)),
                ),
            )
            .select((
                post::all_columns,
                UserSafe::safe_columns_tuple(),
                BoardSafe::safe_columns_tuple(),
                board_user_ban::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                post_saved::all_columns.nullable(),
                post_read::all_columns.nullable(),
                user_block::all_columns.nullable(),
                post_like::score.nullable(),
            ))
            .into_boxed();
        
        
        // THIS FILTER FLIPPING SUCKS (if there are zero records on board_subscriber)

        // if let Some(listing_type) = self.listing_type {
        //     match listing_type {
        //         ListingType::Subscribed => {
        //             query = query.filter(board_subscriber::user_id.is_not_null())
        //         }
        //         ListingType::All => {
        //             query = query.filter(
        //                 board_subscriber::user_id.eq(user_id_join),
        //             )
        //         }
        //     }
        // }

        if let Some(board_id) = self.board_id {
            query = query
                .filter(post::board_id.eq(board_id))
                .then_order_by(post_aggregates::stickied.desc());
        }

        if let Some(url_search) = self.url_search {
            query = query.filter(post::url.eq(url_search));
        }

        if let Some(search_term) = self.search_term {
            let searcher = fuzzy_search(&search_term);
            query = query.filter(
              post::title
                .ilike(searcher.to_owned())
                .or(post::body.ilike(searcher)),
            );
          }

        
        // If query is for specific person show the removed/deleted
        if let Some(creator_id) = self.creator_id {
            query = query.filter(post::creator_id.eq(creator_id));
        }

        if self.saved_only.unwrap_or(false) {
            query = query.filter(post_saved::id.is_not_null());
        }


        query = match self.sort.unwrap_or(SortType::Hot) {
            SortType::Active => query
                .then_order_by(
                    hot_rank(
                        post_aggregates::score,
                        post_aggregates::newest_comment_time,
                    )
                    .desc()
                )
                .then_order_by(post_aggregates::newest_comment_time.desc()),
            SortType::Hot => query
                .then_order_by(hot_rank(post_aggregates::score, post_aggregates::published).desc())
                .then_order_by(post_aggregates::published.desc()),
            SortType::New => query
                .then_order_by(post_aggregates::published.desc()),
            SortType::Old => query
                .then_order_by(post_aggregates::published.asc()),
            SortType::NewComments => query
                .then_order_by(post_aggregates::newest_comment_time.desc()),
            SortType::MostComments => query
                .then_order_by(post_aggregates::comments.desc())
                .then_order_by(post_aggregates::published.desc()),
            SortType::TopAll => query
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::published.desc()),
            SortType::TopYear => query
                .filter(post_aggregates::published.gt(now - 1.years()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::published.desc()),
            SortType::TopMonth => query
                .filter(post_aggregates::published.gt(now - 1.months()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::published.desc()),
            SortType::TopWeek => query
                .filter(post_aggregates::published.gt(now - 1.weeks()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::published.desc()),
            SortType::TopDay => query
                .filter(post_aggregates::published.gt(now - 1.days()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::published.desc()),
        };

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .limit(limit)
            .offset(offset)
            .filter(post::removed.eq(false))
            .filter(post::deleted.eq(false))
            .filter(board::removed.eq(false))
            .filter(board::deleted.eq(false));
        
        debug!("Post Query View: {:?}", debug_query::<Pg, _>(&query));

        let res = query.load::<PostViewTuple>(self.conn)?;

        Ok(PostView::from_tuple_to_vec(res))

    }
}

impl ViewToVec for PostView {
    type DbTuple = PostViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                post: a.0,
                creator: a.1,
                board: a.2,
                creator_banned_from_board: a.3.is_some(),
                counts: a.4,
                subscribed: BoardSubscriber::to_subscribed_type(&a.5),
                saved: a.6.is_some(),
                read: a.7.is_some(),
                creator_blocked: a.8.is_some(),
                my_vote: a.9,
            })
            .collect::<Vec<Self>>()
    }
}