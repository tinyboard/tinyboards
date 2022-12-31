// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, RunQueryDsl};
use std::{thread, time::Duration};
use tinyboards_db::utils::DbPool;
use tinyboards_utils::error::TinyBoardsError;
use tracing::info;

/// Schedules various cleanup tasks for tinyboards in a background thread
pub fn setup(pool: DbPool) -> Result<(), TinyBoardsError> {
    let mut scheduler = Scheduler::new();
    let mut frequent_scheduler = Scheduler::new();

    let mut conn = pool
        .get()
        .map_err(|_| TinyBoardsError::from_message(500, "error getting db pool"))?;
    
    update_banned_when_expired(&mut conn);
    update_user_aggregates_rep(&mut conn);

    // On startup, reindex the tables non-concurrently
    reindex_aggregates_tables(&mut conn, true);
    
    scheduler
    .every(TimeUnits::hour(1)).run(move || {
        update_banned_when_expired(&mut conn);
        reindex_aggregates_tables(&mut conn, true);
    });

    let mut conn2 = pool
        .get()
        .map_err(|_| TinyBoardsError::from_message(500, "error getting db pool"))?;

    frequent_scheduler
    .every(TimeUnits::minutes(5))
    .run(move || {
        update_user_aggregates_rep(&mut conn2);
    });

    // Manually run the scheduler in an event loop
    loop {
        scheduler.run_pending();
        frequent_scheduler.run_pending();
        thread::sleep(Duration::from_millis(1000));
    }
}

/// Reindex the aggregates tables every one hour
/// This is necessary because hot_rank is actually a mutable function
fn reindex_aggregates_tables(conn: &mut PgConnection, concurrently: bool) {
    for table_name in &["post_aggregates", "comment_aggregates", "board_aggregates"] {
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
        "update users set is_banned = false where is_banned = true and unban_date < now()";
    sql_query(update_ban_expires_stmt)
        .execute(conn)
        .expect("update banned when expires");
}

/// update the rep calculation on user_aggregates
fn update_user_aggregates_rep(conn: &mut PgConnection) {
    info!("Updating rep values on user_aggregates ...");
    let update_user_aggregates_rep_stmt = 
    "update user_aggregates ua
     set rep = calc.rep
     from (
         select
             u.id as user_id, 
             round((coalesce(pd.score, 0) + coalesce(cd.score, 0)) / coalesce(pd.posts, 1)) as rep 
         from users u
         left join (
             select p.creator_id,
                 count(distinct p.id) as posts,
                 sum(pv.score) as score
                 from posts p
                 left join post_votes pv on p.id = pv.post_id
                 group by p.creator_id
             ) pd on u.id = pd.creator_id
         left join (
             select c.creator_id,
                 count(distinct c.id) as comments,
                 sum(cv.score) as score
                 from comments c
                 left join comment_votes cv on c.id = cv.comment_id
                 group by c.creator_id
             ) cd on u.id = cd.creator_id
         ) calc 
     where ua.user_id = calc.user_id;";
     sql_query(update_user_aggregates_rep_stmt)
         .execute(conn)
         .expect("update user aggregates rep values");
}