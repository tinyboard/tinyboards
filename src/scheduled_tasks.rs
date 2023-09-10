// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use diesel::{sql_query, PgConnection, Connection, RunQueryDsl};
use std::{thread, time::Duration};
use tinyboards_utils::error::TinyBoardsError;
use tracing::{info, error};

/// Schedules various cleanup tasks for tinyboards in a background thread
pub fn setup(db_url: String) -> Result<(), TinyBoardsError> {
    let mut scheduler = Scheduler::new();
    let mut frequent_scheduler = Scheduler::new();

    let mut conn1 = PgConnection::establish(&db_url)
        .expect("could not establish connection");

    update_banned_when_expired(&mut conn1);
    update_person_aggregates_rep(&mut conn1);

    // On startup, reindex the tables non-concurrently
    reindex_aggregates_tables(&mut conn1, true);

    // On startup, calculate the activity stats
    active_counts(&mut conn1);
    
    scheduler
    .every(TimeUnits::hour(1)).run(move || {
        active_counts(&mut conn1);
        update_banned_when_expired(&mut conn1);
        reindex_aggregates_tables(&mut conn1, true);
    });

    let mut conn2 = PgConnection::establish(&db_url)
        .expect("could not establish connection");

    frequent_scheduler
    .every(TimeUnits::minutes(5))
    .run(move || {
        update_person_aggregates_rep(&mut conn2);
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
    info!("Updating is_banned column if it expired on the person table...");
    let update_ban_expires_stmt =
        "update person set is_banned = false where is_banned = true and unban_date < now()";
    sql_query(update_ban_expires_stmt)
        .execute(conn)
        .expect("update banned when expires");
}

/// update the rep calculation on person_aggregates
fn update_person_aggregates_rep(conn: &mut PgConnection) {
    info!("Updating rep values on person_aggregates ...");
    let update_person_aggregates_rep_stmt = 
    "update person_aggregates pa
     set rep = calc.rep
     from (
         select
             pu.id as person_id, 
             round((coalesce(pd.score, 0) + coalesce(cd.score, 0)) / coalesce(pd.posts, 1)) as rep 
         from person pu
         left join (
             select p.creator_id,
                 count(distinct p.id) as posts,
                 sum(pv.score) as score
                 from posts p
                 left join post_votes pv on p.id = pv.post_id
                 group by p.creator_id
             ) pd on pu.id = pd.creator_id
         left join (
             select c.creator_id,
                 count(distinct c.id) as comments,
                 sum(cv.score) as score
                 from comments c
                 left join comment_votes cv on c.id = cv.comment_id
                 group by c.creator_id
             ) cd on pu.id = cd.creator_id
         ) calc 
     where pa.person_id = calc.person_id;";
     sql_query(update_person_aggregates_rep_stmt)
         .execute(conn)
         .expect("update person aggregates rep values");
}

/// Re-calculate the site and board active counts every 12 hours
fn active_counts(conn: &mut PgConnection) {
    info!("Updating the site and board aggregates with active counts...");

    let intervals = vec![
        ("1 day", "day"),
        ("1 week", "week"),
        ("1 month", "month"),
        ("6 months", "half_year")
    ];

    for i in &intervals {
        let update_site_stmt = format!(
            "update site_aggregates set users_active_{} = (select * from site_aggregates_activity('{}'))",
            i.1,
            i.0
        );
        match sql_query(update_site_stmt).execute(conn) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to update site stats: {}", e)
            }
        }
        let update_board_stmt = format!(
            "update board_aggregates ba set users_active_{} = mv.count_ from board_aggregates_activity('{}') mv where ba.board_id = mv.board_id_",
            i.1,
            i.0
        );
        match sql_query(update_board_stmt).execute(conn) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to update board stats: {}", e)
            }
        }
    }
    info!("Done.")
}