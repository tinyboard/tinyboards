use crate::{newtypes::DbUrl, CommentSortType, SortType};
use diesel::{
    result::{Error as DieselError, Error::QueryBuilderError},
    PgConnection, Connection, sql_types::Text, serialize::{ToSql, Output}, pg::Pg, backend::Backend, deserialize::FromSql,
};
use tinyboards_federation::{fetch::object_id::ObjectId, traits::Object};
use tinyboards_utils::{error::TinyBoardsError, settings::structs::Settings};
use bb8::PooledConnection;
use diesel_async::{
    pg::AsyncPgConnection,
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{env, env::VarError};
use tracing::info;
use url::Url;


pub type DbPool = Pool<AsyncPgConnection>;

pub const DEFAULT_FETCH_LIMIT: i64 = 20;
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

pub fn limit_and_offset_unlimited(page: Option<i64>, limit: Option<i64>) -> (i64, i64) {
    let limit = limit.unwrap_or(DEFAULT_FETCH_LIMIT);
    let offset = limit * (page.unwrap_or(1) - 1);
    (limit, offset)
}

pub fn naive_now() -> chrono::NaiveDateTime {
    chrono::prelude::Utc::now().naive_utc()
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_unpooled_connection() -> PgConnection {
    let db_url = match get_db_url_from_env() {
        Ok(url) => url,
        Err(e) => panic!(
            "Failed to read database URL from env var TINYBOARDS_DATABASE_URL: {}",
            e
        ),
    };

    let mut conn = PgConnection::establish(&db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));

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
            Err(e) => Err(TinyBoardsError::from_error_message(e, 400, "invalid url")),
        },
        None => Ok(None),
    }
}

pub fn diesel_option_overwrite(opt: &Option<String>) -> Option<Option<String>> {
    match opt {
        // empty string is erase
        Some(unwrapped) => {
            if !unwrapped.eq("") {
                Some(Some(unwrapped.clone()))
            } else {
                Some(None)
            }
        }
        None => None,
    }
}

pub fn post_to_comment_sort_type(sort: SortType) -> CommentSortType {
    match sort {
        SortType::Active | SortType::Hot => CommentSortType::Hot,
        SortType::New | SortType::NewComments | SortType::MostComments => CommentSortType::New,
        SortType::Old => CommentSortType::Old,
        SortType::TopDay
        | SortType::TopAll
        | SortType::TopWeek
        | SortType::TopMonth
        | SortType::TopYear => CommentSortType::Top,
    }
}


pub fn get_db_url_from_env() -> Result<String, VarError> {
    env::var("TINYBOARDS_DATABASE_URL")
}

pub fn get_db_url(settings: Option<&Settings>) -> String {
    match get_db_url_from_env() {
        Ok(url) => url,
        Err(e) => match settings {
            Some(settings) => settings.get_database_url(),
            None => panic!("Failed to read database URL from env var TINYBOARDS_DATABASE_URL: {e}"),
        },
    }
}

pub fn run_migrations(db_url: &str) {
    let mut conn = 
        PgConnection::establish(db_url).unwrap_or_else(|e| panic!("Error connecting to {db_url}: {e}"));
    info!("Running db migrations! (this may take a while...)");
    let _ = &mut conn
        .run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| panic!("Couldn't run DB Migrations: {e}"));
    info!("Database migrations complete.")
}

async fn build_db_pool_settings_opt(settings: Option<&Settings>) -> Result<DbPool, TinyBoardsError> {
    let db_url = get_db_url(settings);
    let pool_size = settings.map(|s| s.database.pool_size).unwrap_or(5);
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_url);
    let pool = Pool::builder()
        .max_size(pool_size)
        .min_idle(Some(1))
        .build(manager)
        .await?;

    // If there's no settings then run DB Migrations (unit testing)
    if settings.is_none() {
        run_migrations(&db_url);
    }

    Ok(pool)
}

pub async fn build_db_pool(settings: &Settings) -> Result<DbPool, TinyBoardsError> {
    build_db_pool_settings_opt(Some(settings)).await
}

pub async fn build_db_pool_for_tests() -> DbPool {
    build_db_pool_settings_opt(None)
        .await
        .expect("db pool missing")
}

pub async fn get_conn(
    pool: &DbPool,
) -> Result<PooledConnection<AsyncDieselConnectionManager<AsyncPgConnection>>, DieselError> {
    pool.get().await.map_err(|e| QueryBuilderError(e.into()))
}

impl ToSql<Text, Pg> for DbUrl {
    fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
      <std::string::String as ToSql<Text, Pg>>::to_sql(&self.0.to_string(), &mut out.reborrow())
    }
  }
  
  impl<DB: Backend> FromSql<Text, DB> for DbUrl
  where
    String: FromSql<Text, DB>,
  {
    fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
      let str = String::from_sql(value)?;
      Ok(DbUrl(Box::new(Url::parse(&str)?)))
    }
  }
  
  impl<Kind> From<ObjectId<Kind>> for DbUrl
  where
    Kind: Object + Send + 'static,
    for<'de2> <Kind as Object>::Kind: serde::Deserialize<'de2>,
  {
    fn from(id: ObjectId<Kind>) -> Self {
      DbUrl(Box::new(id.into()))
    }
  }