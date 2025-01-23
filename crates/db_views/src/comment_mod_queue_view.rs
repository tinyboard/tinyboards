use std::collections::HashMap;

use crate::{structs::{CommentView}};
use diesel::{dsl::*, result::Error, *};
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    models::{
        board::board_subscriber::BoardSubscriber,
        board::board_person_bans::BoardPersonBan,
        board::boards::BoardSafe,
        comment::comments::Comment,
        comment::comment_saved::CommentSaved,
        post::posts::Post,
        person::person_blocks::*,
        person::person::*,
    },
    schema::{
        board_subscriber, board_person_bans, boards, comment_aggregates, comment_votes, comments, comment_report,
        posts, person_blocks, person_board_blocks, comment_saved, person,
    },
    traits::{ToSafe},
    utils::{limit_and_offset_unlimited, get_conn, DbPool},
};

use typed_builder::TypedBuilder;
use diesel_async::RunQueryDsl;

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

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentModQuery<'a> {
    #[builder(!default)]
    pool: &'a DbPool,
    my_person_id: i32,
    page: Option<i64>,
    limit: Option<i64>,
}

#[derive(Default, Clone)]
pub struct CommentModQueryResponse {
    pub comments: Vec<CommentView>,
    pub count: i64,
}

impl<'a> CommentModQuery<'a> {
    pub async fn list(self) -> Result<CommentModQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;
        use diesel::dsl::*;

        let person_id_join = self.my_person_id;
        // let local_user_id_join = self.my_person_id;

        let query = comments::table
            .inner_join(person::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .inner_join(comment_report::table.on(comment_report::comment_id.eq(comments::id).and(comment_report::resolved.eq(false))))
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

        let count_query = comments::table
            .inner_join(person::table)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(comment_aggregates::table)
            .inner_join(comment_report::table.on(comment_report::comment_id.eq(comments::id).and(comment_report::resolved.eq(false))))
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
                person_board_blocks::table.on(boards::id
                    .eq(person_board_blocks::board_id)
                    .and(person_board_blocks::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_votes::table.on(comments::id
                    .eq(comment_votes::comment_id)
                    .and(comment_votes::person_id.eq(person_id_join))),
            )
            /*.left_join(
                local_user_language::table.on(
                    comments::language_id
                        .eq(local_user_language::language_id)
                        .and(local_user_language::local_user_id.eq(local_user_id_join))
                )
            )*/
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

        let (limit, offset) = limit_and_offset_unlimited(self.page, self.limit);

        let res = query
            .limit(limit)
            .offset(offset)
            .load::<CommentViewTuple>(conn)
            .await?;

        let comments = Self::load_report_counts(res, self.pool).await?;
        let count = count_query.count().get_result::<i64>(conn)
            .await?;

        Ok(CommentModQueryResponse { comments, count })
    }

    async fn load_report_counts(items: Vec<CommentViewTuple>, pool: &DbPool) -> Result<Vec<CommentView>, Error> {
        let conn = &mut get_conn(pool).await?;

        let ids: Vec<i32> = items.iter().map(|c| c.0.id).collect();

        let counts_query = comments::table
            .filter(comments::id.eq_any(ids))
            .inner_join(comment_report::table.on(comment_report::comment_id.eq(comments::id).and(comment_report::resolved.eq(false))))
            .group_by(comments::id)
            .select((comments::id, count(comment_report::id)))
            .load::<(i32, i64)>(conn)
            .await?;

        let mut map: HashMap<i32, i64> = HashMap::new();

        for (comment_id, report_count) in counts_query {
            map.insert(comment_id, report_count);
        }

        Ok(
            items.into_iter().map(|a| {
                let cid = a.0.id;
        
                CommentView {
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
                    report_count: map.remove(&cid)
                }
            }).collect()
        )
    }
}
