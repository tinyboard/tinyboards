use crate::{structs::{LocalUserView, PostView}, DeleteableOrRemoveable};
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::PostAggregates,
    models::{
        board::board_subscriber::BoardSubscriber,
        board::board_person_bans::BoardPersonBan,
        board::boards::BoardSafe,
        post::posts::Post,
        post::post_read::PostRead,
        post::post_saved::PostSaved,
        person::person_blocks::PersonBlock,
        person::person::*,
        person::local_user::*,
    },
    schema::{
        board_subscriber, board_person_bans, boards, post_aggregates, post_votes, posts,
        person_blocks, person_board_blocks, post_read, post_saved, person, post_report
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, fuzzy_search, limit_and_offset, get_conn, DbPool},
    ListingType, SortType,
};
use typed_builder::TypedBuilder;

type PostViewTuple = (
    Post,
    PersonSafe,
    BoardSafe,
    Option<BoardPersonBan>,
    PostAggregates,
    Option<BoardSubscriber>,
    Option<PostSaved>,
    Option<PostRead>,
    Option<PersonBlock>,
    Option<i16>,
);
use diesel_async::RunQueryDsl;

sql_function!(fn coalesce(x: sql_types::Nullable<sql_types::BigInt>, y: sql_types::BigInt) -> sql_types::BigInt);

impl PostView {
    pub async fn read(
        pool: &DbPool,
        post_id: i32,
        my_person_id: Option<i32>,
        is_mod_or_admin: Option<bool>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let person_id_join = my_person_id.unwrap_or(-1);
        let query = posts::table
            .find(post_id)
            .inner_join(person::table)
            .inner_join(boards::table)
            .left_join(
                board_person_bans::table.on(posts::board_id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(posts::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriber::table.on(posts::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                post_saved::table.on(posts::id
                    .eq(post_saved::post_id)
                    .and(post_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                post_read::table.on(posts::id
                    .eq(post_read::post_id)
                    .and(post_read::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(posts::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                post_votes::table.on(posts::id
                    .eq(post_votes::post_id)
                    .and(post_votes::person_id.eq(person_id_join))),
            )
            .select((
                posts::all_columns,
                PersonSafe::safe_columns_tuple(),
                BoardSafe::safe_columns_tuple(),
                board_person_bans::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                post_saved::all_columns.nullable(),
                post_read::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                post_votes::score.nullable(),
            ))
            .into_boxed();
        
        // hide deleted or removed posts from non-admin or mods
        // we prolly don't want this here though...
        /*if !is_mod_or_admin.unwrap_or(true) {
            query = query
                .filter(posts::is_deleted.eq(false))
                .filter(posts::is_removed.eq(false));
        }*/

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
            post_vote,
        ) = query.first::<PostViewTuple>(conn).await?;

        let my_vote = if my_person_id.is_some() && post_vote.is_none() {
            Some(0)
        } else {
            post_vote
        };

        let is_mod_or_admin = is_mod_or_admin.unwrap_or(false);
        let report_count = if is_mod_or_admin {
            let count = post_report::table
                .filter(post_report::post_id.eq(post_id))
                .filter(post_report::resolved.eq(false))
                .select(count(post_report::id))
                .first::<i64>(conn).await;

            match count {
                Ok(count) => Some(count),
                Err(_) => None
            }
        } else {
            None
        };

        Ok(PostView {
            post,
            creator: Some(creator),
            board,
            creator_banned_from_board: creator_banned_from_board.is_some(),
            counts,
            subscribed: BoardSubscriber::to_subscribed_type(&subscriber),
            saved: saved.is_some(),
            read: read.is_some(),
            creator_blocked: creator_blocked.is_some(),
            my_vote,
            report_count
        })
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    listing_type: Option<ListingType>,
    sort: Option<SortType>,
    creator_id: Option<i32>,
    board_id: Option<i32>,
    show_deleted: Option<bool>,
    show_removed: Option<bool>,
    search_term: Option<String>,
    url_search: Option<String>,
    saved_only: Option<bool>,
    user: Option<&'a LocalUser>,
    show_nsfw: Option<bool>,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct PostQueryResponse {
    pub posts: Vec<PostView>,
    pub count: i64,
}

impl<'a> PostQuery<'a> {
    pub async fn list(self) -> Result<PostQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        use diesel::dsl::*;

        let person_id_join = match self.user {
            Some(user) => user.person_id,
            None => -1,
        };

        let mut query = posts::table
            .inner_join(person::table)
            .inner_join(boards::table)
            .left_join(
                board_person_bans::table.on(posts::board_id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(posts::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriber::table.on(posts::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                post_saved::table.on(posts::id
                    .eq(post_saved::post_id)
                    .and(post_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                post_read::table.on(posts::id
                    .eq(post_read::post_id)
                    .and(post_read::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(posts::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                post_votes::table.on(posts::id
                    .eq(post_votes::post_id)
                    .and(post_votes::person_id.eq(person_id_join))),
            )
            .select((
                posts::all_columns,
                PersonSafe::safe_columns_tuple(),
                BoardSafe::safe_columns_tuple(),
                board_person_bans::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                post_saved::all_columns.nullable(),
                post_read::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                post_votes::score.nullable(),
            ))
            .into_boxed();

        let mut count_query = posts::table
            .inner_join(person::table)
            .inner_join(boards::table)
            .left_join(
                board_person_bans::table.on(posts::board_id
                    .eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(posts::creator_id))
                    .and(
                        board_person_bans::expires
                            .is_null()
                            .or(board_person_bans::expires.gt(now)),
                    )),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriber::table.on(posts::board_id
                    .eq(board_subscriber::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                post_saved::table.on(posts::id
                    .eq(post_saved::post_id)
                    .and(post_saved::person_id.eq(person_id_join))),
            )
            .left_join(
                post_read::table.on(posts::id
                    .eq(post_read::post_id)
                    .and(post_read::person_id.eq(person_id_join))),
            )
            .left_join(
                person_blocks::table.on(posts::creator_id
                    .eq(person_blocks::target_id)
                    .and(person_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                post_votes::table.on(posts::id
                    .eq(post_votes::post_id)
                    .and(post_votes::person_id.eq(person_id_join))),
            )
            .select((posts::all_columns,))
            .into_boxed();

        if let Some(listing_type) = self.listing_type {
            match listing_type {
                ListingType::Subscribed => {
                    query = query.filter(board_subscriber::person_id.is_not_null())
                }
                ListingType::All => {
                    query = query.filter(
                        boards::is_hidden
                            .eq(false)
                            .or(board_subscriber::person_id.eq(person_id_join)),
                    )
                },
                ListingType::Local => {
                    query = query.filter(
                        boards::local
                            .eq(true),
                    )
                }
            }
        }

        if !self.show_deleted.unwrap_or(false) {
            query = query
                .filter(posts::is_deleted.eq(false));

            count_query = count_query.filter(posts::is_deleted.eq(false));
        }

        if !self.show_removed.unwrap_or(false) {
            query = query
                .filter(posts::is_removed.eq(false));

            count_query = count_query.filter(posts::is_removed.eq(false));
        }

        if let Some(board_id) = self.board_id {
            query = query
                .filter(posts::board_id.eq(board_id))
                .then_order_by(post_aggregates::is_stickied.desc());
        }

        if let Some(url_search) = self.url_search {
            let url_searcher = fuzzy_search(&url_search);
            query = query.filter(posts::url.ilike(url_searcher.to_owned()));
        }

        if let Some(search_term) = self.search_term {
            let searcher = fuzzy_search(&search_term);
            query = query.filter(
                posts::title
                    .ilike(searcher.to_owned())
                    .or(posts::body.ilike(searcher)),
            );
        }

        // If query is for specific person show the removed/deleted
        if let Some(creator_id) = self.creator_id {
            query = query.filter(posts::creator_id.eq(creator_id));

            count_query = count_query.filter(posts::creator_id.eq(creator_id));
        }

        if self.saved_only.unwrap_or(false) {
            query = query.filter(post_saved::id.is_not_null());
        }

        // if show_nsfw is NOT TRUE, then we filter is_nsfw posts from query
        if !self.show_nsfw.unwrap_or(false) {
            query = query.filter(posts::is_nsfw.eq(false));
        }

        // filter posts from blocked boards and person
        if self.user.is_some() {
            query = query.filter(person_board_blocks::person_id.is_null());
            query = query.filter(person_blocks::person_id.is_null());
        }

        // sticky posts on top
        query = query.then_order_by(post_aggregates::is_stickied.desc());

        query = match self.sort.unwrap_or(SortType::Hot) {
            SortType::Active => query
                .then_order_by(
                    hot_rank(post_aggregates::score, post_aggregates::newest_comment_time).desc(),
                )
                .then_order_by(post_aggregates::newest_comment_time.desc()),
            SortType::Hot => query
                .then_order_by(
                    hot_rank(post_aggregates::score, post_aggregates::creation_date).desc(),
                )
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::New => query.then_order_by(post_aggregates::creation_date.desc()),
            SortType::Old => query.then_order_by(post_aggregates::creation_date.asc()),
            SortType::NewComments => {
                query.then_order_by(post_aggregates::newest_comment_time.desc())
            }
            SortType::MostComments => query
                .then_order_by(post_aggregates::comments.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopAll => query
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopYear => query
                .filter(post_aggregates::creation_date.gt(now - 1.years()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopMonth => query
                .filter(post_aggregates::creation_date.gt(now - 1.months()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopWeek => query
                .filter(post_aggregates::creation_date.gt(now - 1.weeks()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopDay => query
                .filter(post_aggregates::creation_date.gt(now - 1.days()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
        };


        if let Some(_id) = self.board_id {
            query = query
                .then_order_by(posts::featured_local.desc())
                .then_order_by(posts::featured_board.desc());
        } else {
            query = query
                .then_order_by(posts::featured_local.desc());

        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .limit(limit)
            .offset(offset)
            .filter(boards::is_banned.eq(false))
            .filter(boards::is_deleted.eq(false));

        let res = query.load::<PostViewTuple>(conn).await?;

        let posts = PostView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(conn).await?;

        Ok(PostQueryResponse { posts, count })
    }
}

impl DeleteableOrRemoveable for PostView {
    fn hide_if_removed_or_deleted(&mut self, user_view: Option<&LocalUserView>) {
        /*if !(self.post.is_deleted || self.post.is_removed) {
            return self;
        }*/

        if let Some(user_view) = user_view {
            // admins see everything
            if user_view.local_user.is_admin {
                return;
            }

            // person can see their own removed content
            if self.post.is_removed && user_view.person.id == self.post.creator_id {
                return;
            }
        }

        let obscure_text: String = {
            if self.post.is_deleted {
                "[ deleted by author ]"
            } else {
                "[ removed by mod ]"
            }
        }
        .into();

        self.post.title = obscure_text.clone();
        self.post.body = obscure_text.clone();
        self.post.body_html = obscure_text;
        self.post.url = None;
        self.post.image = None;
        self.post.creator_id = -1;
        self.creator = None;
    }
}

impl ViewToVec for PostView {
    type DbTuple = PostViewTuple;
    fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
        items
            .into_iter()
            .map(|a| Self {
                post: a.0,
                creator: Some(a.1),
                board: a.2,
                creator_banned_from_board: a.3.is_some(),
                counts: a.4,
                subscribed: BoardSubscriber::to_subscribed_type(&a.5),
                saved: a.6.is_some(),
                read: a.7.is_some(),
                creator_blocked: a.8.is_some(),
                my_vote: a.9,
                report_count: None
            })
            .collect::<Vec<Self>>()
    }
}