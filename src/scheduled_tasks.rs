// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use diesel::sql_query;
use std::{thread, time::Duration};
use tinyboards_db::utils::{DbPool, get_conn};
use tinyboards_utils::error::TinyBoardsError;
use tracing::info;
use diesel_async::RunQueryDsl;

/// Schedules various cleanup tasks for tinyboards in a background thread
pub async fn setup(pool: DbPool) -> Result<(), TinyBoardsError> {
    let mut scheduler = Scheduler::new();
    let mut frequent_scheduler = Scheduler::new();

    update_banned_when_expired(&pool).await;
    update_user_aggregates_rep(&pool).await;

    // On startup, reindex the tables non-concurrently
    reindex_aggregates_tables(&pool, true);
    
    scheduler
    .every(TimeUnits::hour(1)).run(move || {
        update_banned_when_expired(&pool);
        reindex_aggregates_tables(&pool, true);
    });
    
    frequent_scheduler
    .every(TimeUnits::minutes(5))
    .run(move || {
        update_user_aggregates_rep(&pool);
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
async fn reindex_aggregates_tables(pool: &DbPool, concurrently: bool) {
    
    for table_name in &["post_aggregates", "comment_aggregates", "board_aggregates"] {
        reindex_table(pool, table_name, concurrently).await;
    }
}

async fn reindex_table(pool: &DbPool, table_name: &str, concurrently: bool) {
    let conn = &mut get_conn(pool).await?;
    let concurrently_str = if concurrently { "concurrently" } else { "" };
    info!("Reindexing table {} {} ...", concurrently_str, table_name);
    let query = format!("reindex table {} {}", concurrently_str, table_name);
    sql_query(query).execute(conn).await.expect("reindex table");
    info!("Done.");
}

/// Set banned to false after ban expires
async fn update_banned_when_expired(pool: &DbPool) {
    let conn = &mut get_conn(pool).await?;
    info!("Updating banned column if it expires ...");
    let update_ban_expires_stmt =
        "update users set is_banned = false where is_banned = true and unban_date < now()";
    sql_query(update_ban_expires_stmt)
        .execute(conn)
        .await
        .expect("update banned when expires");
}

/// update the rep calculation on user_aggregates
async fn update_user_aggregates_rep(pool: &DbPool) {
    let conn = &mut get_conn(pool).await?;
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
         .await
         .expect("update user aggregates rep values");
}