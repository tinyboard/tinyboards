use crate::structs::CommentReportView;
use diesel::{
    dsl::now,
    result::Error,
    BoolExpressionMethods,
    ExpressionMethods,
    JoinOnDsl,
    NullableExpressionMethods,
    QueryDsl
};
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    aggregates::structs::CommentAggregates,
    schema::{
        comments,
        comment_aggregates,
        comment_votes,
        comment_report,
        boards,
        board_mods,
        board_person_bans,
        person,
        posts,
    },
    models::{
        comment::comments::Comment,
        comment::comment_report::CommentReport,
        board::boards::Board,
        board::board_person_bans::BoardPersonBan,
        person::person::PersonSafe,
        post::posts::Post,
    },
    traits::{JoinView, ToSafe},
    utils::{get_conn, limit_and_offset, DbPool},
};
use typed_builder::TypedBuilder;


impl CommentReportView {
    /// returns the CommentReportView for the provided report_id
    pub async fn read(
        pool: &DbPool,
        report_id: i32,
        my_person_id: Option<i32>,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

        let person_id_join = my_person_id.unwrap_or(-1);

        let res = comment_report::table
            .find(report_id)
            .inner_join(comments::table)
            .inner_join(posts::table.on(comments::board_id.eq(posts::board_id)))
            .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
            .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
            .inner_join(person_alias_1.on(comments::creator_id.eq(person_alias_1.field(person::id))))
            .inner_join(
                comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id))
            )
            .left_join(
                board_person_bans::table.on(
                    boards::id
                        .eq(board_person_bans::board_id)
                        .and(board_person_bans::person_id.eq(comments::creator_id))
                )
            )
            .left_join(
                comment_votes::table.on(
                    comments::id
                        .eq(comment_votes::comment_id)
                        .and(comment_votes::person_id.eq(person_id_join))
                )
            )
            .left_join(
                person_alias_2
                    .on(comment_report::resolver_id.eq(person_alias_2.field(person::id).nullable()))
            )
            .select((
                comment_report::all_columns,
                comments::all_columns,
                posts::all_columns,
                boards::all_columns,
                PersonSafe::safe_columns_tuple(),
                person_alias_1.fields(PersonSafe::safe_columns_tuple()),
                comment_aggregates::all_columns,
                board_person_bans::all_columns.nullable(),
                comment_votes::score.nullable(),
                person_alias_2.fields(PersonSafe::safe_columns_tuple()).nullable(),
            ))
            .first::<<CommentReportView as JoinView>::JoinTuple>(conn)
            .await?;

        Ok(Self::from_tuple(res))
    }

    /// Returns the current unresolved post report count for the communities you mod
    pub async fn get_report_count(
        pool: &DbPool,
        my_person_id: i32,
        admin: bool,
        board_id: Option<i32>,
    ) -> Result<i64, Error> {
        use diesel::dsl::count;

        let conn = &mut get_conn(pool).await?;

        let mut query = comment_report::table
        .inner_join(comments::table)
        .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
        .filter(comment_report::resolved.eq(false))
        .into_boxed();

        if let Some(board_id) = board_id {
        query = query.filter(posts::board_id.eq(board_id))
        }

        // If its not an admin, get only the ones you mod
        if !admin {
        query
            .inner_join(
            board_mods::table.on(
                board_mods::board_id
                .eq(posts::board_id)
                .and(board_mods::person_id.eq(my_person_id)),
            ),
            )
            .select(count(comment_report::id))
            .first::<i64>(conn)
            .await
        } else {
        query
            .select(count(comment_report::id))
            .first::<i64>(conn)
            .await
        }
    }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentReportQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  #[builder(!default)]
  my_person_id: i32,
  #[builder(!default)]
  admin: bool,
  comment_id: Option<i32>,
  board_id: Option<i32>,
  page: Option<i64>,
  limit: Option<i64>,
  unresolved_only: Option<bool>,
}

#[derive(Default, Clone)]
pub struct CommentReportQueryResponse {
  pub reports: Vec<CommentReportView>,
  pub count: i64,
}

impl<'a> CommentReportQuery<'a> {
    pub async fn list(self) -> Result<CommentReportQueryResponse, Error> {
      let conn = &mut get_conn(self.pool).await?;
  
      let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);
  
      let query_ = comment_report::table
        .inner_join(comments::table)
        .inner_join(posts::table.on(comments::post_id.eq(posts::id)))
        .inner_join(boards::table.on(posts::board_id.eq(boards::id)))
        .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
        .inner_join(person_alias_1.on(comments::creator_id.eq(person_alias_1.field(person::id))))
        .inner_join(
          comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id)),
        )
        .left_join(
          board_person_bans::table.on(
            boards::id
              .eq(board_person_bans::board_id)
              .and(board_person_bans::person_id.eq(comments::creator_id))
              .and(
                board_person_bans::expires
                  .is_null()
                  .or(board_person_bans::expires.gt(now)),
              ),
          ),
        )
        .left_join(
          comment_votes::table.on(
            comments::id
              .eq(comment_votes::comment_id)
              .and(comment_votes::person_id.eq(self.my_person_id)),
          ),
        )
        .left_join(
          person_alias_2
            .on(comment_report::resolver_id.eq(person_alias_2.field(person::id).nullable())),
        )
        .select((
          comment_report::all_columns,
          comments::all_columns,
          posts::all_columns,
          boards::all_columns,
          PersonSafe::safe_columns_tuple(),
          person_alias_1.fields(PersonSafe::safe_columns_tuple()),
          comment_aggregates::all_columns,
          board_person_bans::all_columns.nullable(),
          comment_votes::score.nullable(),
          person_alias_2.fields(PersonSafe::safe_columns_tuple()).nullable(),
        ));

      let mut query = query_.clone().into_boxed();  
      let mut count_query = query_.clone().into_boxed();
        
      if let Some(comment_id) = self.comment_id {
        query = query.filter(comments::id.eq(comment_id));
        count_query = count_query.filter(comments::id.eq(comment_id));
      }
        
      if let Some(board_id) = self.board_id {
        query = query.filter(posts::board_id.eq(board_id));
        count_query = count_query.filter(posts::board_id.eq(board_id));
      }
  
      if self.unresolved_only.unwrap_or(false) {
        query = query.filter(comment_report::resolved.eq(false));
        count_query = count_query.filter(comment_report::resolved.eq(false));
      }
  
      let (limit, offset) = limit_and_offset(self.page, self.limit)?;
  
      query = query
        .order_by(comment_report::creation_date.desc())
        .limit(limit)
        .offset(offset);

      count_query = count_query
        .limit(limit)
        .offset(offset);

      // If its not an admin, get only the ones you mod
      if !self.admin {
        let res = query
          .inner_join(
            board_mods::table.on(
              board_mods::board_id
                .eq(posts::board_id)
                .and(board_mods::person_id.eq(self.my_person_id)),
            ),
          )
          .load::<<CommentReportView as JoinView>::JoinTuple>(conn)
          .await?;

        let count = count_query
          .inner_join(
            board_mods::table.on(
              board_mods::board_id
                .eq(posts::board_id)
                .and(board_mods::person_id.eq(self.my_person_id)),
            ),
          )
          .count().get_result::<i64>(conn)
          .await?;

        let reports = res.into_iter().map(CommentReportView::from_tuple).collect();

        Ok(CommentReportQueryResponse{ reports, count })
      } else {
        let res = query
          .load::<<CommentReportView as JoinView>::JoinTuple>(conn)
          .await?;
        let count = count_query.count().get_result::<i64>(conn).await?;
        let reports = res.into_iter().map(CommentReportView::from_tuple).collect();
        Ok(CommentReportQueryResponse { reports, count })
      }
    }
}

impl JoinView for CommentReportView {
    type JoinTuple = (
      CommentReport,
      Comment,
      Post,
      Board,
      PersonSafe,
      PersonSafe,
      CommentAggregates,
      Option<BoardPersonBan>,
      Option<i16>,
      Option<PersonSafe>,
    );
  
    fn from_tuple(a: Self::JoinTuple) -> Self {
      Self {
        data: a.0,
        comment: a.1,
        post: a.2,
        board: a.3,
        creator: a.4,
        comment_creator: a.5,
        counts: a.6,
        creator_banned_from_board: a.7.is_some(),
        my_vote: a.8,
        resolver: a.9,
      }
    }
  }