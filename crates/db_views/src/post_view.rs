use crate::{structs::PostView, DeleteableOrRemoveable};
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::PostAggregates,
    models::{
        board::board_subscriptions::BoardSubscriber,
        board::board_user_bans::BoardUserBan,
        board::boards::BoardSafe,
        post::posts::Post,
        post::user_post_read::PostRead,
        post::user_post_save::PostSaved,
        //user::user::User,
        user::{
            user_blocks::UserBlock,
            users::{User, UserSafe},
        },
    },
    schema::{
        board_subscriptions, board_user_bans, boards, post_aggregates, post_votes, posts,
        user_blocks, user_board_blocks, user_post_read, user_post_save, users,
    },
    traits::{ToSafe, ViewToVec},
    utils::{functions::hot_rank, fuzzy_search, limit_and_offset},
    ListingType, SortType,
};
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
            post_vote,
        ) = posts::table
            .find(post_id)
            .inner_join(users::table)
            .inner_join(boards::table)
            .left_join(
                board_user_bans::table.on(posts::board_id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(posts::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_post_save::table.on(posts::id
                    .eq(user_post_save::post_id)
                    .and(user_post_save::user_id.eq(user_id_join))),
            )
            .left_join(
                user_post_read::table.on(posts::id
                    .eq(user_post_read::post_id)
                    .and(user_post_read::user_id.eq(user_id_join))),
            )
            .left_join(
                user_blocks::table.on(posts::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                post_votes::table.on(posts::id
                    .eq(post_votes::post_id)
                    .and(post_votes::user_id.eq(user_id_join))),
            )
            .select((
                posts::all_columns,
                UserSafe::safe_columns_tuple(),
                BoardSafe::safe_columns_tuple(),
                board_user_bans::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriptions::all_columns.nullable(),
                user_post_save::all_columns.nullable(),
                user_post_read::all_columns.nullable(),
                user_blocks::all_columns.nullable(),
                post_votes::score.nullable(),
            ))
            .first::<PostViewTuple>(conn)?;

        let my_vote = if my_user_id.is_some() && post_vote.is_none() {
            Some(0)
        } else {
            post_vote
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
    show_deleted_or_removed: Option<bool>,
    search_term: Option<String>,
    url_search: Option<String>,
    saved_only: Option<bool>,
    user: Option<&'a User>,
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
    pub fn list(self) -> Result<PostQueryResponse, Error> {
        use diesel::dsl::*;

        let user_id_join = match self.user {
            Some(user) => user.id,
            None => -1,
        };

        let mut query = posts::table
            .inner_join(users::table)
            .inner_join(boards::table)
            .left_join(
                board_user_bans::table.on(posts::board_id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(posts::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_post_save::table.on(posts::id
                    .eq(user_post_save::post_id)
                    .and(user_post_save::user_id.eq(user_id_join))),
            )
            .left_join(
                user_post_read::table.on(posts::id
                    .eq(user_post_read::post_id)
                    .and(user_post_read::user_id.eq(user_id_join))),
            )
            .left_join(
                user_blocks::table.on(posts::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                user_board_blocks::table.on(boards::id
                    .eq(user_board_blocks::board_id)
                    .and(user_board_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                post_votes::table.on(posts::id
                    .eq(post_votes::post_id)
                    .and(post_votes::user_id.eq(user_id_join))),
            )
            .select((
                posts::all_columns,
                UserSafe::safe_columns_tuple(),
                BoardSafe::safe_columns_tuple(),
                board_user_bans::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriptions::all_columns.nullable(),
                user_post_save::all_columns.nullable(),
                user_post_read::all_columns.nullable(),
                user_blocks::all_columns.nullable(),
                post_votes::score.nullable(),
            ))
            .into_boxed();

        let count_query = posts::table
            .inner_join(users::table)
            .inner_join(boards::table)
            .left_join(
                board_user_bans::table.on(posts::board_id
                    .eq(board_user_bans::board_id)
                    .and(board_user_bans::user_id.eq(posts::creator_id))
                    .and(
                        board_user_bans::expires
                            .is_null()
                            .or(board_user_bans::expires.gt(now)),
                    )),
            )
            .inner_join(post_aggregates::table)
            .left_join(
                board_subscriptions::table.on(posts::board_id
                    .eq(board_subscriptions::board_id)
                    .and(board_subscriptions::user_id.eq(user_id_join))),
            )
            .left_join(
                user_post_save::table.on(posts::id
                    .eq(user_post_save::post_id)
                    .and(user_post_save::user_id.eq(user_id_join))),
            )
            .left_join(
                user_post_read::table.on(posts::id
                    .eq(user_post_read::post_id)
                    .and(user_post_read::user_id.eq(user_id_join))),
            )
            .left_join(
                user_blocks::table.on(posts::creator_id
                    .eq(user_blocks::target_id)
                    .and(user_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                user_board_blocks::table.on(boards::id
                    .eq(user_board_blocks::board_id)
                    .and(user_board_blocks::user_id.eq(user_id_join))),
            )
            .left_join(
                post_votes::table.on(posts::id
                    .eq(post_votes::post_id)
                    .and(post_votes::user_id.eq(user_id_join))),
            )
            .select((posts::all_columns,))
            .into_boxed();

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
        }

        if !self.show_deleted_or_removed.unwrap_or(false) {
            query = query
                .filter(posts::is_removed.eq(false))
                .filter(posts::is_deleted.eq(false));
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
        }

        if self.saved_only.unwrap_or(false) {
            query = query.filter(user_post_save::id.is_not_null());
        }

        // if show_nsfw is NOT TRUE, then we filter is_nsfw posts from query
        if !self.show_nsfw.unwrap_or(false) {
            query = query.filter(posts::is_nsfw.eq(false));
        }

        // filter posts from blocked boards and users
        if self.user.is_some() {
            query = query.filter(user_board_blocks::user_id.is_null());
            query = query.filter(user_blocks::user_id.is_null());
        }

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

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .limit(limit)
            .offset(offset)
            .filter(posts::is_removed.eq(false))
            .filter(posts::is_deleted.eq(false))
            .filter(boards::is_banned.eq(false))
            .filter(boards::is_deleted.eq(false));

        let res = query.load::<PostViewTuple>(self.conn)?;

        let posts = PostView::from_tuple_to_vec(res);
        let count = count_query.count().get_result::<i64>(self.conn)?;

        Ok(PostQueryResponse { posts, count })
    }
}

impl DeleteableOrRemoveable for PostView {
    fn hide_if_removed_or_deleted(&mut self, user: Option<&User>) {
        /*if !(self.post.is_deleted || self.post.is_removed) {
            return self;
        }*/

        if let Some(user) = user {
            // admins see everything
            if user.is_admin {
                return;
            }

            // users can see their own removed content
            if self.post.is_removed && user.id == self.post.creator_id {
                return;
            }
        }

        let obscure_text: String = {
            if self.post.is_deleted {
                "[ retracted ]"
            } else {
                "[ purged ]"
            }
        }
        .into();

        self.post.title = obscure_text.clone();
        self.post.body = obscure_text.clone();
        self.post.body_html = obscure_text;
        self.post.url = None;
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
            })
            .collect::<Vec<Self>>()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::post_view::{PostQuery, PostView};
//     use diesel::PgConnection;
//     use tinyboards_db::{
//         aggregates::structs::PostAggregates,
//         utils::establish_unpooled_connection,
//         models::{
//             board::boards::*,
//             board::user_board_blocks::{BoardBlock, BoardBlockForm},
//             site::site::{Site, SiteForm},
//             user::user::{User, UserForm},
//             user::user_blocks::{UserBlock, UserBlockForm},
//             post::posts::*,
//         },
//         traits::{Blockable, Crud, Voteable}, SortType,
//     };
//     use serial_test::serial;

//     struct Data {
//         inserted_site: Site,
//         inserted_user: User,
//         inserted_blocked_user: User,
//         inserted_board: Board,
//         inserted_post: Post,
//     }

//     fn init_data(conn: &mut PgConnection) -> Data {

//         let inserted_site_form = SiteForm {
//             name: Some("domain.tld".to_string()),
//             description: Some("my heckin website".to_string()),
//             ..SiteForm::default()
//         };

//         let inserted_site = Site::create(conn, &inserted_site_form).unwrap();

//         let inserted_user_form = UserForm {
//             name: "kroner".to_string(),
//             passhash: "the_most_secure_password".to_string(),
//             email: Some("example@domain.tld".to_string()),
//             ..UserForm::default()
//         };

//         let inserted_user = User::create(conn, &inserted_user_form).unwrap();

//         let inserted_blocked_user_form = UserForm {
//             name: "bullyhunter05".to_string(),
//             passhash: "the_most_secure_password2".to_string(),
//             email: Some("example@domain.tld".to_string()),
//             ..UserForm::default()
//         };

//         let inserted_blocked_user = User::create(conn, &inserted_blocked_user_form).unwrap();

//         let inserted_board_form = BoardForm {
//             name: Some("Test Board".to_string()),
//             creator_id: Some(inserted_user.id.clone()),
//             ..BoardForm::default()
//         };

//         let inserted_board = Board::create(conn, &inserted_board_form).unwrap();

//         let inserted_post_form = PostForm {
//             title: "test post".to_string(),
//             type_: Some("text".to_string()),
//             body: Some("this is a test post lol".to_string()),
//             creator_id: inserted_user.id.clone(),
//             board_id: inserted_board.id.clone(),
//             ..PostForm::default()
//         };

//         let inserted_post = Post::create(conn, &inserted_post_form).unwrap();

//         // also create a post from blocked user
//         let inserted_post_blocked_user_form = PostForm {
//             title: "test post 2 (should not appear)".to_string(),
//             type_: Some("text".to_string()),
//             body: Some("this is a test post lol".to_string()),
//             creator_id: inserted_blocked_user.id.clone(),
//             board_id: inserted_board.id.clone(),
//             ..PostForm::default()
//         };

//         Post::create(conn, &inserted_post_blocked_user_form).unwrap();

//         Data {
//             inserted_site,
//             inserted_user,
//             inserted_blocked_user,
//             inserted_board,
//             inserted_post,
//         }
//     }

//     #[test]
//     #[serial]
//     fn post_listing_with_user() {
//         let conn = &mut establish_unpooled_connection();
//         let data = init_data(conn);

//         let read_post_listing = PostQuery::builder()
//             .conn(conn)
//             .sort(Some(SortType::New))
//             .board_id(Some(data.inserted_board.id))
//             .user_id(Some(data.inserted_user.id))
//             .build()
//             .list()
//             .unwrap();

//         let post_listing_single_with_user =
//             PostView::read(conn, data.inserted_post.id, Some(data.inserted_user.id)).unwrap();

//         let mut expected_post_listing_with_user = todo!()

//     }
// }
