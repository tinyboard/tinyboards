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

    // On startup, reindex the tables non-concurrently
    reindex_aggregates_tables(&mut conn1, true);

    // On startup, calculate the activity stats
    active_counts(&mut conn1);

    // On startup, update post and comment rankings
    update_post_rankings(&mut conn1);
    update_comment_rankings(&mut conn1);

    // On startup, update user aggregates
    update_user_aggregates(&mut conn1);

    // On startup, update site aggregates
    update_site_aggregates(&mut conn1);

    scheduler
    .every(TimeUnits::hour(1)).run(move || {
        active_counts(&mut conn1);
        reindex_aggregates_tables(&mut conn1, true);
        update_site_aggregates(&mut conn1);
    });

    let mut conn3 = PgConnection::establish(&db_url)
        .expect("could not establish connection");

    scheduler
    .every(TimeUnits::minutes(10))
    .run(move || {
        update_post_rankings(&mut conn3);
        update_comment_rankings(&mut conn3);
        update_user_aggregates(&mut conn3);
    });

    let mut conn2 = PgConnection::establish(&db_url)
        .expect("could not establish connection");

    frequent_scheduler
    .every(TimeUnits::minutes(5))
    .run(move || {
        update_banned_when_expired(&mut conn2);
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
    info!("Updating is_banned column if it expired on the user table...");
    let update_ban_expires_stmt =
        "update users set is_banned = false where is_banned = true and unban_date < now()";
    sql_query(update_ban_expires_stmt)
        .execute(conn)
        .expect("update banned when expires");
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

/// Update hot_rank and controversy_rank for posts every 10 minutes
fn update_post_rankings(conn: &mut PgConnection) {
    info!("Updating post rankings...");

    // Update hot_rank based on score and creation date
    let update_hot_rank_stmt =
        "UPDATE post_aggregates SET hot_rank = hot_rank(score, creation_date)";

    // Update hot_rank_active based on score and newest comment time
    let update_hot_rank_active_stmt =
        "UPDATE post_aggregates SET hot_rank_active = hot_rank(score, newest_comment_time)";

    // Update controversy_rank based on upvotes, downvotes, and creation date
    let update_controversy_rank_stmt =
        "UPDATE post_aggregates SET controversy_rank = controversy_rank(upvotes, downvotes, creation_date)";

    match sql_query(update_hot_rank_stmt).execute(conn) {
        Ok(_) => info!("Updated post hot_rank"),
        Err(e) => error!("Failed to update post hot_rank: {}", e)
    }

    match sql_query(update_hot_rank_active_stmt).execute(conn) {
        Ok(_) => info!("Updated post hot_rank_active"),
        Err(e) => error!("Failed to update post hot_rank_active: {}", e)
    }

    match sql_query(update_controversy_rank_stmt).execute(conn) {
        Ok(_) => info!("Updated post controversy_rank"),
        Err(e) => error!("Failed to update post controversy_rank: {}", e)
    }

    info!("Done updating post rankings.");
}

/// Update hot_rank and controversy_rank for comments every 10 minutes
fn update_comment_rankings(conn: &mut PgConnection) {
    info!("Updating comment rankings...");

    // Update comment hot_rank based on score and creation date
    let update_hot_rank_stmt =
        "UPDATE comment_aggregates SET hot_rank = hot_rank(score, creation_date)";

    // Update comment controversy_rank based on upvotes, downvotes, and creation date
    let update_controversy_rank_stmt =
        "UPDATE comment_aggregates SET controversy_rank = controversy_rank(upvotes, downvotes, creation_date)";

    match sql_query(update_hot_rank_stmt).execute(conn) {
        Ok(_) => info!("Updated comment hot_rank"),
        Err(e) => error!("Failed to update comment hot_rank: {}", e)
    }

    match sql_query(update_controversy_rank_stmt).execute(conn) {
        Ok(_) => info!("Updated comment controversy_rank"),
        Err(e) => error!("Failed to update comment controversy_rank: {}", e)
    }

    info!("Done updating comment rankings.");
}

/// Update user aggregates (post count, comment count, scores) every hour
fn update_user_aggregates(conn: &mut PgConnection) {
    info!("Updating user aggregates...");

    // Recalculate all user aggregates to ensure data integrity
    let update_user_aggregates_stmt = r#"
        INSERT INTO user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
        SELECT
            u.id as user_id,
            COALESCE(p.post_count, 0) as post_count,
            COALESCE(ps.post_score, 0) as post_score,
            COALESCE(c.comment_count, 0) as comment_count,
            COALESCE(cs.comment_score, 0) as comment_score
        FROM users u
        LEFT JOIN (
            SELECT creator_id, COUNT(*) as post_count
            FROM posts
            GROUP BY creator_id
        ) p ON u.id = p.creator_id
        LEFT JOIN (
            SELECT posts.creator_id, SUM(post_aggregates.score) as post_score
            FROM posts
            JOIN post_aggregates ON posts.id = post_aggregates.post_id
            GROUP BY posts.creator_id
        ) ps ON u.id = ps.creator_id
        LEFT JOIN (
            SELECT creator_id, COUNT(*) as comment_count
            FROM comments
            GROUP BY creator_id
        ) c ON u.id = c.creator_id
        LEFT JOIN (
            SELECT comments.creator_id, SUM(comment_aggregates.score) as comment_score
            FROM comments
            JOIN comment_aggregates ON comments.id = comment_aggregates.comment_id
            GROUP BY comments.creator_id
        ) cs ON u.id = cs.creator_id
        ON CONFLICT (user_id) DO UPDATE SET
            post_count = EXCLUDED.post_count,
            post_score = EXCLUDED.post_score,
            comment_count = EXCLUDED.comment_count,
            comment_score = EXCLUDED.comment_score;
    "#;

    match sql_query(update_user_aggregates_stmt).execute(conn) {
        Ok(rows_affected) => info!("Updated user aggregates for {} users", rows_affected),
        Err(e) => error!("Failed to update user aggregates: {}", e)
    }

    info!("Done updating user aggregates.");
}

/// Update site aggregates (total users, posts, comments, boards, upvotes, downvotes)
fn update_site_aggregates(conn: &mut PgConnection) {
    info!("Updating site aggregates...");

    // Recalculate site-wide statistics
    let update_site_aggregates_stmt = r#"
        UPDATE site_aggregates SET
            users = (SELECT COUNT(*) FROM users WHERE is_deleted = false AND is_banned = false),
            posts = (SELECT COUNT(*) FROM posts WHERE is_deleted = false AND is_removed = false),
            comments = (SELECT COUNT(*) FROM comments WHERE is_deleted = false AND is_removed = false),
            boards = (SELECT COUNT(*) FROM boards WHERE is_deleted = false AND is_removed = false),
            upvotes = (SELECT COALESCE(SUM(upvotes), 0) FROM post_aggregates) + (SELECT COALESCE(SUM(upvotes), 0) FROM comment_aggregates),
            downvotes = (SELECT COALESCE(SUM(downvotes), 0) FROM post_aggregates) + (SELECT COALESCE(SUM(downvotes), 0) FROM comment_aggregates)
        WHERE site_id = 1;
    "#;

    match sql_query(update_site_aggregates_stmt).execute(conn) {
        Ok(_) => info!("Updated site aggregates"),
        Err(e) => error!("Failed to update site aggregates: {}", e)
    }

    info!("Done updating site aggregates.");
}