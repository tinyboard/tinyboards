use diesel::{
    //backend::Backend,
    //deserialize::FromSql,
    //pg::Pg,
    result::Error::QueryBuilderError,
    //serialize::{Output, ToSql},
    //sql_types::Text,
    //Connection,
    //PgConnection,
};

const FETCH_LIMIT_DEFAULT: i64 = 10;
pub const FETCH_LIMIT_MAX: i64 = 50;

pub mod functions {
    use diesel::sql_types::*;
  
    diesel::sql_function! { fn hot_rank(score: BigInt, time: Timestamp) -> Integer; }
  
    diesel::sql_function!(fn lower(x: Text) -> Text);
}

pub fn fuzzy_search(q: &str) -> String {
    let replaced = q.replace('%', "\\%").replace('_', "\\_").replace(' ', "%");
    format!("%{}%", replaced)
}

pub fn limit_and_offset(
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<(i64, i64), diesel::result::Error> {
    let page = match page {
      Some(page) => {
        if page < 1 {
          return Err(QueryBuilderError("Page is < 1".into()));
        } else {
          page
        }
      }
      None => 1,
    };
    let limit = match limit {
      Some(limit) => {
        if !(1..=FETCH_LIMIT_MAX).contains(&limit) {
          return Err(QueryBuilderError(
            format!("Fetch limit is > {}", FETCH_LIMIT_MAX).into(),
          ));
        } else {
          limit
        }
      }
      None => FETCH_LIMIT_DEFAULT,
    };
    let offset = limit * (page - 1);
    Ok((limit, offset))
}