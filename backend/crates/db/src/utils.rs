use diesel::{
    result::Error::QueryBuilderError,
    sql_query,
    Connection, PgConnection,
};
use diesel_async::RunQueryDsl;
use bb8::PooledConnection;
use diesel_async::{
    pg::AsyncPgConnection,
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{env, env::VarError};
use tinyboards_utils::{error::TinyBoardsError, settings::structs::Settings};
use tracing::info;
use uuid::Uuid;

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

pub fn now_utc() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

#[derive(diesel::QueryableByName)]
struct BanStatus {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_banned: bool,
}

/// Check and update ban status for a specific user if their ban has expired
pub async fn check_and_update_person_ban_status(pool: &DbPool, user_id: Uuid) -> Result<bool, TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    let update_ban_expires_stmt =
        "UPDATE users SET is_banned = false WHERE id = $1 AND is_banned = true AND unban_date < now() RETURNING is_banned";

    let result: Result<BanStatus, diesel::result::Error> = sql_query(update_ban_expires_stmt)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result(conn)
        .await;

    match result {
        Ok(ban_status) => Ok(ban_status.is_banned),
        Err(diesel::result::Error::NotFound) => {
            let check_ban_stmt = "SELECT is_banned FROM users WHERE id = $1";
            let ban_status: BanStatus = sql_query(check_ban_stmt)
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .get_result(conn)
                .await?;
            Ok(ban_status.is_banned)
        },
        Err(e) => Err(TinyBoardsError::Database(format!("Failed to check ban status: {}", e))),
    }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_unpooled_connection() -> PgConnection {
    let db_url = match get_db_url_from_env() {
        Ok(url) => url,
        Err(e) => panic!(
            "Failed to read database URL from env var TINYBOARDS_DATABASE_URL or DATABASE_URL: {}",
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

pub fn diesel_option_overwrite(opt: &Option<String>) -> Option<Option<String>> {
    match opt {
        Some(unwrapped) => {
            if !unwrapped.is_empty() {
                Some(Some(unwrapped.clone()))
            } else {
                Some(None)
            }
        }
        None => None,
    }
}

pub fn get_db_url_from_env() -> Result<String, VarError> {
    env::var("TINYBOARDS_DATABASE_URL").or_else(|_| env::var("DATABASE_URL"))
}

pub fn get_db_url(settings: Option<&Settings>) -> String {
    match get_db_url_from_env() {
        Ok(url) => url,
        Err(e) => match settings {
            Some(settings) => settings.get_database_url(),
            None => panic!("Failed to read database URL from env var TINYBOARDS_DATABASE_URL or DATABASE_URL: {e}"),
        },
    }
}

pub fn run_migrations(db_url: &str) {
    let mut conn = PgConnection::establish(db_url)
        .unwrap_or_else(|e| panic!("Error connecting to {db_url}: {e}"));
    info!("Running db migrations! (this may take a while...)");
    let _ = &mut conn
        .run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| panic!("Couldn't run DB Migrations: {e}"));
    info!("Database migrations complete.")
}

async fn build_db_pool_settings_opt(
    settings: Option<&Settings>,
) -> Result<DbPool, TinyBoardsError> {
    let db_url = get_db_url(settings);
    let pool_size = settings.map(|s| s.database.pool_size).unwrap_or(5);
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_url);
    let pool = Pool::builder()
        .max_size(pool_size)
        .min_idle(Some(1))
        .build(manager)
        .await
        .map_err(|e| TinyBoardsError::Database(format!("Failed to build pool: {}", e)))?;

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
) -> Result<PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>, TinyBoardsError> {
    pool.get().await.map_err(|e| TinyBoardsError::Database(format!("Pool error: {}", e)))
}
