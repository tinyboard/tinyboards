use diesel::{
  // backend::Backend,
  // deserialize::FromSql,
  // pg::Pg,
  result::Error::QueryBuilderError,
  // serialize::{Output, ToSql},
  // sql_types::Text,
  Connection,
  PgConnection,
};
use diesel_migrations::{EmbeddedMigrations, embed_migrations, MigrationHarness};
use url::Url;
use crate::newtypes::DbUrl;
use tinyboards_utils::error::TinyBoardsError;



pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn get_database_url_from_env() -> Result<String, std::env::VarError> {
  std::env::var("TINYBOARDS_DATABASE_URL")
}

const DEFAULT_FETCH_LIMIT: i64 = 20;
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
      None => DEFAULT_FETCH_LIMIT,
    };
    let offset = limit * (page - 1);
    Ok((limit, offset))
}

pub fn limit_and_offset_unlimited(
  page: Option<i64>,
  limit: Option<i64>,
) -> (i64, i64) {
  let limit = limit.unwrap_or(DEFAULT_FETCH_LIMIT);
  let offset = limit * (page.unwrap_or(1) - 1);
  (limit, offset)
} 

pub fn naive_now() -> chrono::NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_unpooled_connection() -> PgConnection {
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(e) => panic!(
      "Failed to read database URL from env var TINYBOARDS_DATABASE_URL: {}",
      e
    ),
  };

  let mut conn = 
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
  
  let _ = &mut conn
    .run_pending_migrations(MIGRATIONS)
    .unwrap_or_else(|_| panic!("Couldn't run DB Migrations"));

  conn
}

pub fn diesel_option_overwrite_to_url(
  opt: &Option<String>,
) -> Result<Option<Option<DbUrl>>, TinyBoardsError> {
  match opt.as_ref().map(std::string::String::as_str) {
    //empty string = erase
    Some("") => Ok(Some(None)),
    Some(str_url) => match Url::parse(str_url) {
      Ok(url) => Ok(Some(Some(url.into()))),
      Err(_e) => Err(TinyBoardsError::from_string("invalid url", 500)),
    },
    None => Ok(None)
  }
}

pub fn diesel_option_overwrite(
  opt: &Option<String>
) -> Option<Option<String>> {
  match opt {
    // empty string is erase
    Some(unwrapped) => {
      if !unwrapped.eq("") {
        Some(Some(unwrapped.clone()))
      } else {
        Some(None)
      }
    },
    None => None,
  }
}