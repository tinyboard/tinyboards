use std::collections::HashMap;

use diesel::dsl::count;
use tinyboards_db::models::person::person_blocks::PersonBlock;
use tinyboards_db::models::post::post_read::PostRead;
use tinyboards_db::models::post::post_saved::PostSaved;
use tinyboards_db::schema::person_aggregates;
use tinyboards_db::{
    aggregates::structs::PersonAggregates, models::board::board_subscriber::BoardSubscriber,
};

use crate::structs::PostView;
use diesel::dsl::now;
use tinyboards_db::models::board::boards::BoardSafe;

use diesel::{
    result::Error, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    QueryDsl,
};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    aggregates::structs::PostAggregates,
    models::{
        board::board_person_bans::BoardPersonBan, person::person::PersonSafe, post::posts::Post,
    },
    schema::{
        board_mods, board_person_bans, board_subscriber, boards, person, person_blocks,
        person_board_blocks, post_aggregates, post_read, post_report, post_saved, post_votes,
        posts,
    },
    traits::ToSafe,
    utils::{get_conn, limit_and_offset, DbPool},
};
use typed_builder::TypedBuilder;

type PostViewTuple = (
    Post,
    PersonSafe,
    PersonAggregates,
    BoardSafe,
    Option<BoardPersonBan>,
    PostAggregates,
    Option<BoardSubscriber>,
    Option<PostSaved>,
    Option<PostRead>,
    Option<PersonBlock>,
    Option<i16>,
    //Option<BoardModerator>,
    Option<i32>,
);

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostModQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    #[builder(!default)]
    my_person_id: i32,
    #[builder(!default)]
    admin: bool,
    post_id: Option<i32>,
    board_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
    unresolved_only: Option<bool>,
}

#[derive(Default, Clone)]
pub struct PostModQueryResponse {
    pub posts: Vec<PostView>,
    pub count: i64,
}

impl<'a> PostModQuery<'a> {
    /*fn from_tuple_to_vec(items: Vec<PostViewTuple>) -> Vec<PostView> {
        items.into_iter()
            .map(|p| PostView {

            })
    }*/

    pub async fn list(self) -> Result<PostModQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;

        let person_id_join = self.my_person_id;

        // let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

        let query_ = posts::table
            .inner_join(person::table)
            .inner_join(person_aggregates::table.on(person_aggregates::person_id.eq(person::id)))
            .inner_join(boards::table)
            .inner_join(
                post_report::table.on(posts::id
                    .eq(post_report::post_id)
                    .and(post_report::resolved.eq(false))),
            )
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
            .left_join(
                board_mods::table.on(posts::board_id
                    .eq(board_mods::board_id)
                    .and(board_mods::person_id.eq(person_id_join))
                    .and(board_mods::invite_accepted.eq(true))),
            )
            .select((
                posts::all_columns,
                PersonSafe::safe_columns_tuple(),
                person_aggregates::all_columns,
                BoardSafe::safe_columns_tuple(),
                board_person_bans::all_columns.nullable(),
                post_aggregates::all_columns,
                board_subscriber::all_columns.nullable(),
                post_saved::all_columns.nullable(),
                post_read::all_columns.nullable(),
                person_blocks::all_columns.nullable(),
                post_votes::score.nullable(),
                board_mods::permissions.nullable(),
            ));

        let mut query = query_.clone().into_boxed();
        let mut count_query = query_.clone().into_boxed();

        if let Some(post_id) = self.post_id {
            query = query.filter(posts::id.eq(post_id));
        }

        if let Some(board_id) = self.board_id {
            query = query.filter(posts::board_id.eq(board_id));
            count_query = count_query.filter(posts::board_id.eq(board_id));
        }

        if self.unresolved_only.unwrap_or(false) {
            query = query.filter(post_report::resolved.eq(false));
            count_query = count_query.filter(post_report::resolved.eq(false));
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit)?;

        query = query
            .order_by(posts::creation_date.desc())
            .limit(limit)
            .offset(offset);

        count_query = count_query.limit(limit).offset(offset);

        if !self.admin {
            /*let res = query
            .inner_join(
                board_mods::table.on(board_mods::board_id
                    .eq(posts::board_id)
                    .and(board_mods::person_id.eq(self.my_person_id))),
            )
            .load::<PostViewTuple>(conn)
            .await?;*/
            let res = query
                .filter(board_mods::person_id.eq(person_id_join))
                .load::<PostViewTuple>(conn)
                .await?;
            let posts = Self::load_report_counts(res, self.pool).await?;
            /*let count = count_query
            .inner_join(
                board_mods::table.on(board_mods::board_id
                    .eq(posts::board_id)
                    .and(board_mods::person_id.eq(self.my_person_id))),
            )
            .count()
            .get_result::<i64>(conn)
            .await?;*/
            let count = count_query
                .filter(board_mods::person_id.eq(person_id_join))
                .count()
                .get_result::<i64>(conn)
                .await?;

            Ok(PostModQueryResponse { posts, count })
        } else {
            let res = query.load::<PostViewTuple>(conn).await?;
            let posts = Self::load_report_counts(res, self.pool).await?;
            let count = count_query.count().get_result::<i64>(conn).await?;

            Ok(PostModQueryResponse { posts, count })
        }
    }

    async fn load_report_counts(
        items: Vec<PostViewTuple>,
        pool: &DbPool,
    ) -> Result<Vec<PostView>, Error> {
        let conn = &mut get_conn(pool).await?;

        let ids: Vec<i32> = items.iter().map(|p| p.0.id).collect();

        let counts_query = posts::table
            .filter(posts::id.eq_any(ids))
            .inner_join(
                post_report::table.on(post_report::post_id
                    .eq(posts::id)
                    .and(post_report::resolved.eq(false))),
            )
            .group_by(posts::id)
            .select((posts::id, count(post_report::id)))
            .load::<(i32, i64)>(conn)
            .await?;

        let mut map: HashMap<i32, i64> = HashMap::new();

        for (post_id, report_count) in counts_query {
            map.insert(post_id, report_count);
        }

        Ok(items
            .into_iter()
            .map(|a| {
                let pid = a.0.id;

                PostView {
                    post: a.0,
                    creator: Some(a.1),
                    creator_counts: Some(a.2),
                    board: a.3,
                    creator_banned_from_board: a.4.is_some(),
                    counts: a.5,
                    subscribed: BoardSubscriber::to_subscribed_type(&a.6),
                    saved: a.7.is_some(),
                    read: a.8.is_some(),
                    creator_blocked: a.9.is_some(),
                    my_vote: a.10,
                    report_count: map.remove(&pid),
                    mod_permissions: a.11,
                }
            })
            .collect())
    }
}
