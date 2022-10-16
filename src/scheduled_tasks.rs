// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use porpl_db::utils::DbPool;
use porpl_utils::error::PorplError;
use std::{thread, time::Duration};
use tracing::info;

/// Schedules various cleanup tasks for porpl in a background thread
pub fn setup(pool: DbPool) -> Result<(), PorplError> {
  let mut scheduler = Scheduler::new();

  let mut conn = pool.get()
    .map_err(|_| PorplError::from_string("error getting db pool", 500))?;
  update_banned_when_expired(&mut conn);

  // On startup, reindex the tables non-concurrently
  reindex_aggregates_tables(&mut conn, true);
  scheduler.every(1.hour()).run(move || {
    update_banned_when_expired(&mut conn);
    reindex_aggregates_tables(&mut conn, true);
  });


  // Manually run the scheduler in an event loop
  loop {
    scheduler.run_pending();
    thread::sleep(Duration::from_millis(1000));
  }
}

/// Reindex the aggregates tables every one hour
/// This is necessary because hot_rank is actually a mutable function
fn reindex_aggregates_tables(conn: &mut PgConnection, concurrently: bool) {
  for table_name in &[
    "post_aggregates",
    "comment_aggregates",
    "board_aggregates",
  ] {
    reindex_table(conn, table_name, concurrently);
  }
}

fn reindex_table(conn: &mut PgConnection, table_name: &str, concurrently: bool) {
  let concurrently_str = if concurrently { "concurrently" } else { "" };
  info!("Reindexing table {} {} ...", concurrently_str, table_name);
  let query = format!("reindex table {} {}", concurrently_str, table_name);
  sql_query(query).execute(conn).expect("reindex table");
  info!("Done.");
}

/// Set banned to false after ban expires
fn update_banned_when_expired(conn: &mut PgConnection) {
  info!("Updating banned column if it expires ...");
  let update_ban_expires_stmt =
    "update user_ set banned = false where banned = true and expires < now()";
  sql_query(update_ban_expires_stmt)
    .execute(conn)
    .expect("update banned when expires");
}