use crate::structs::PostReportView;
use diesel::{
    result::Error,
    BoolExpressionMethods,
    ExpressionMethods,
    JoinOnDsl,
    NullableExpressionMethods,
    QueryDsl, dsl::count,
};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    aggregates::structs::PostAggregates,
    schema::{
        boards,
        board_mods,
        board_person_bans,
        person,
        posts,
        post_aggregates,
        post_votes,
        post_report,
    },
    models::{
        board::boards::Board,
        board::board_person_bans::BoardPersonBan,
        person::person::PersonSafe,
        post::posts::Post,
        post::post_report::PostReport,
    },
    traits::{JoinView, ToSafe},
    utils::{get_conn, limit_and_offset, DbPool},
};
use typed_builder::TypedBuilder;

type PostReportViewTuple = (
    PostReport,
    Post,
    Board,
    PersonSafe,
    PersonSafe,
    Option<BoardPersonBan>,
    Option<i16>,
    PostAggregates,
    Option<PersonSafe>,
);

impl PostReportView {
    /// Returns the PostReportView for the provided report_id
    pub async fn read(
        pool: &DbPool,
        report_id: i32,
        my_person_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

        let person_id_join = my_person_id.unwrap_or(-1);

        let (
            post_report,
            post,
            board,
            creator,
            post_creator,
            creator_banned_from_board,
            post_vote,
            counts,
            resolver
        ) = post_report::table
            .find(report_id)
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
            .inner_join(person_alias_1.on(posts::creator_id.eq(person_alias_1.field(person::id))))
            .left_join(
                board_person_bans::table.on(
                    posts::board_id.eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(posts::creator_id))
                )
            )
            .left_join(
                post_votes::table.on(
                    posts::id
                        .eq(post_votes::post_id)
                        .and(post_votes::person_id.eq(person_id_join))
                )
            )
            .inner_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
            .left_join(
                person_alias_2.on(post_report::resolver_id.eq(person_alias_2.field(person::id).nullable()))
            )
            .select((
                post_report::all_columns,
                posts::all_columns,
                boards::all_columns,
                PersonSafe::safe_columns_tuple(),
                person_alias_1.fields(PersonSafe::safe_columns_tuple()),
                board_person_bans::all_columns.nullable(),
                post_votes::score.nullable(),
                post_aggregates::all_columns,
                person_alias_2.fields(PersonSafe::safe_columns_tuple().nullable())
            ))
            .first::<PostReportViewTuple>(conn)
            .await?;

        let my_vote = post_vote;

        Ok(Self {
            data: post_report,
            post,
            board,
            creator,
            post_creator,
            creator_banned_from_board: creator_banned_from_board.is_some(),
            my_vote,
            counts,
            resolver,
        })
    }

    /// returns current unresolved post report count for the boards you mod
    pub async fn get_report_count(
        pool: &DbPool,
        my_person_id: i32,
        admin: bool,
        board_id: Option<i32>,
    ) -> Result<i64, Error> {
        let conn = &mut get_conn(pool).await?;
        let mut query = post_report::table
            .inner_join(posts::table)
            .filter(post_report::resolved.eq(false))
            .into_boxed();

        if let Some(board_id) = board_id {
            query = query.filter(posts::board_id.eq(board_id));
        }

        if !admin {
            query
                .inner_join(
                    board_mods::table.on(
                        board_mods::board_id
                            .eq(posts::board_id)
                            .and(board_mods::person_id.eq(my_person_id))
                    )
                )
                .select(count(post_report::id))
                .first::<i64>(conn)
                .await
        } else {
            query
                .select(count(post_report::id))
                .first::<i64>(conn)
                .await
        }
    }
}


#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostReportQuery<'a> {
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
pub struct PostReportQueryResponse {
    pub reports: Vec<PostReportView>,
    pub count: i64,
}

impl<'a> PostReportQuery<'a> {
    pub async fn list(self) -> Result<PostReportQueryResponse, Error> {
        let conn = &mut get_conn(self.pool).await?;

        let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

        let query_ = post_report::table
            .inner_join(posts::table)
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
            .inner_join(person_alias_1.on(posts::creator_id.eq(person_alias_1.field(person::id))))
            .left_join(
                board_person_bans::table.on(
                    posts::board_id.eq(board_person_bans::board_id)
                    .and(board_person_bans::person_id.eq(posts::creator_id))
                )
            )
            .left_join(
                post_votes::table.on(
                    posts::id
                        .eq(post_votes::post_id)
                        .and(post_votes::person_id.eq(self.my_person_id))
                )
            )
            .inner_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
            .left_join(
                person_alias_2.on(post_report::resolver_id.eq(person_alias_2.field(person::id).nullable()))
            )
            .select((
                post_report::all_columns,
                posts::all_columns,
                boards::all_columns,
                PersonSafe::safe_columns_tuple(),
                person_alias_1.fields(PersonSafe::safe_columns_tuple()),
                board_person_bans::all_columns.nullable(),
                post_votes::score.nullable(),
                post_aggregates::all_columns,
                person_alias_2.fields(PersonSafe::safe_columns_tuple().nullable())
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
            .order_by(post_report::creation_date.desc())
            .limit(limit)
            .offset(offset);

        count_query = count_query
            .limit(limit)
            .offset(offset);

        if !self.admin {
            let res = query
                .inner_join(
                    board_mods::table.on(
                        board_mods::board_id
                            .eq(posts::board_id)
                            .and(board_mods::person_id.eq(self.my_person_id))
                    )
                )
                .load::<PostReportViewTuple>(conn)
                .await?;
            let reports = res.into_iter().map(PostReportView::from_tuple).collect();
            let count = count_query
                .inner_join(
                    board_mods::table.on(
                        board_mods::board_id
                            .eq(posts::board_id)
                            .and(board_mods::person_id.eq(self.my_person_id))
                    )
                )
                .count()
                .get_result::<i64>(conn)
                .await?;

            Ok(PostReportQueryResponse { reports, count })    
        } else {
            let res = query.load::<PostReportViewTuple>(conn).await?;
            let reports = res.into_iter().map(PostReportView::from_tuple).collect();
            let count = count_query.count().get_result::<i64>(conn).await?;
            
            Ok(PostReportQueryResponse { reports, count })
        }
    }
}

impl JoinView for PostReportView {
    type JoinTuple = PostReportViewTuple;
    fn from_tuple(a: Self::JoinTuple) -> Self {
      Self {
        data: a.0,
        post: a.1,
        board: a.2,
        creator: a.3,
        post_creator: a.4,
        creator_banned_from_board: a.5.is_some(),
        my_vote: a.6,
        counts: a.7,
        resolver: a.8,
      }
    }
  }