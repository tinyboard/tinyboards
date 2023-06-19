use crate::{
    schema::post_report::dsl::{post_report, resolved, resolver_id, updated},
    models::post::post_report::{PostReport, PostReportForm},
    traits::Reportable,
    utils::{get_conn, naive_now, DbPool},
};
use async_trait::async_trait;
use diesel::{
    dsl::{insert_into, update},
    ExpressionMethods,
    QueryDsl,
  };
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

#[async_trait]
impl Reportable for PostReport {
  type Form = PostReportForm;
  type IdType = i32;

  async fn report(pool: &DbPool, report_form: &Self::Form) -> Result<Self, TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_report)
        .values(report_form)
        .get_result::<Self>(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not report post"))
  }

  async fn resolve(pool: &DbPool, report_id_: i32, by_resolver_id: i32) -> Result<usize, TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;
    update(post_report.find(report_id_))
        .set((
            resolved.eq(true),
            resolver_id.eq(by_resolver_id),
            updated.eq(naive_now())
        ))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not resolve post report"))
  }

  async fn unresolve(pool: &DbPool, report_id_: i32, by_resolver_id: i32) -> Result<usize, TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;
    update(post_report.find(report_id_))
        .set((
            resolved.eq(false),
            resolver_id.eq(by_resolver_id),
            updated.eq(naive_now())
        ))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unresolve post report"))
  }
}