// Scheduler, and trait for .seconds(), .minutes(), etc.
use clokwerk::{Scheduler, TimeUnits};
// Import week days and WeekDay
use chrono::{Datelike, NaiveDate};
use diesel::{sql_query, PgConnection, Connection, RunQueryDsl};
use std::{thread, time::Duration};
use tinyboards_utils::error::TinyBoardsError;
use tracing::{info, error};

/// Schedules various cleanup tasks for tinyboards in a background thread
pub fn setup(db_url: String) -> Result<(), TinyBoardsError> {
    let mut scheduler = Scheduler::new();
    let mut frequent_scheduler = Scheduler::new();

    let mut conn1 = PgConnection::establish(&db_url)
        .map_err(|e| TinyboardsError::from_message(500, e.to_string()));

    update_banned_when_expired(&mut conn1);

    // On startup, reindex the tables non-concurrently
    reindex_aggregates_tables(&mut conn1, true);

    // On startup, calculate the activity stats
    active_counts(&mut conn1);

    // On startup, update post and comment rankings
    update_post_rankings(&mut conn1);
    update_comment_rankings(&mut conn1);

    // On startup, initialize post aggregates (triggers will handle real-time updates)
    update_post_aggregates(&mut conn1);

    // On startup, initialize comment aggregates (triggers will handle real-time updates)
    update_comment_aggregates(&mut conn1);

    // On startup, initialize user aggregates (triggers will handle real-time updates)
    update_user_aggregates(&mut conn1);

    // On startup, initialize board aggregates (triggers will handle real-time updates)
    update_board_aggregates(&mut conn1);

    // On startup, initialize site aggregates (triggers will handle real-time updates)
    update_site_aggregates(&mut conn1);

    // On startup, update flair aggregates rolling windows
    update_flair_aggregates(&mut conn1);

    // On startup, clean up expired sessions and password resets
    cleanup_expired_sessions(&mut conn1);
    cleanup_expired_password_resets(&mut conn1);

    // On startup, clean up expired bans
    cleanup_expired_board_bans(&mut conn1);
    cleanup_expired_user_bans(&mut conn1);

    scheduler
    .every(TimeUnits::hour(1)).run(move || {
        active_counts(&mut conn1);
        reindex_aggregates_tables(&mut conn1, true);
        update_flair_aggregates(&mut conn1);
    });

    let mut conn3 = PgConnection::establish(&db_url)
        .map_err(|e| TinyboardsError::from_message(500, e.to_string()));

    scheduler
    .every(TimeUnits::minutes(10))
    .run(move || {
        update_post_rankings(&mut conn3);
        update_comment_rankings(&mut conn3);
    });

    let mut conn2 = PgConnection::establish(&db_url)
        .map_err(|e| TinyboardsError::from_message(500, e.to_string()));

    frequent_scheduler
    .every(TimeUnits::minutes(5))
    .run(move || {
        update_banned_when_expired(&mut conn2);
        cleanup_expired_board_bans(&mut conn2);
        cleanup_expired_user_bans(&mut conn2);
    });

    let mut conn4 = PgConnection::establish(&db_url)
        .map_err(|e| TinyboardsError::from_message(500, e.to_string()));

    // Hourly cleanup of expired sessions, password resets, and old notifications
    scheduler
    .every(TimeUnits::hour(1))
    .run(move || {
        cleanup_expired_sessions(&mut conn4);
        cleanup_expired_password_resets(&mut conn4);
        cleanup_old_read_notifications(&mut conn4);
    });

    let mut conn5 = PgConnection::establish(&db_url)
        .map_err(|e| TinyboardsError::from_message(500, e.to_string()));

    // On startup, ensure next month's partitions exist
    ensure_partitions(&mut conn5);

    // Daily: ensure next month's partitions exist ahead of time
    scheduler
    .every(TimeUnits::day(1))
    .run(move || {
        ensure_partitions(&mut conn5);
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

    match sql_query(query).execute(conn) {
        Ok(_) => info!("Done."),
        Err(e) => error!("Failed to reindex table {}: {}", table_name, e),
    }
}

/// Set banned to false after ban expires
fn update_banned_when_expired(conn: &mut PgConnection) {
    info!("Updating is_banned column if it expired on the user table...");
    let update_ban_expires_stmt =
        "update users set is_banned = false where is_banned = true and unban_date < now()";

    match sql_query(update_ban_expires_stmt).execute(conn) {
        Ok(_) => info!("Done."),
        Err(e) => error!("Failed to update banned column {}: {}", table_name, e),
    }

}

/// Remove expired board-level bans
fn cleanup_expired_board_bans(conn: &mut PgConnection) {
    let stmt = "DELETE FROM board_user_bans WHERE expires_at IS NOT NULL AND expires_at < now()";
    match sql_query(stmt).execute(conn) {
        Ok(count) => {
            if count > 0 {
                info!("Removed {} expired board bans", count);
            }
        }
        Err(e) => error!("Failed to clean up expired board bans: {}", e)
    }
}

/// Remove expired site-level bans
fn cleanup_expired_user_bans(conn: &mut PgConnection) {
    let stmt = "DELETE FROM user_bans WHERE expires_at IS NOT NULL AND expires_at < now()";
    match sql_query(stmt).execute(conn) {
        Ok(count) => {
            if count > 0 {
                info!("Removed {} expired user bans", count);
            }
        }
        Err(e) => error!("Failed to clean up expired user bans: {}", e)
    }
}

/// Remove expired auth sessions
fn cleanup_expired_sessions(conn: &mut PgConnection) {
    let stmt = "DELETE FROM auth_sessions WHERE expires_at < now()";
    match sql_query(stmt).execute(conn) {
        Ok(count) => {
            if count > 0 {
                info!("Removed {} expired auth sessions", count);
            }
        }
        Err(e) => error!("Failed to clean up expired sessions: {}", e)
    }
}

/// Remove expired password reset tokens
fn cleanup_expired_password_resets(conn: &mut PgConnection) {
    let stmt = "DELETE FROM password_resets WHERE expires_at < now()";
    match sql_query(stmt).execute(conn) {
        Ok(count) => {
            if count > 0 {
                info!("Removed {} expired password reset tokens", count);
            }
        }
        Err(e) => error!("Failed to clean up expired password resets: {}", e)
    }
}

/// Delete read notifications older than 90 days
fn cleanup_old_read_notifications(conn: &mut PgConnection) {
    let stmt = "DELETE FROM notifications WHERE is_read = true AND created_at < now() - INTERVAL '90 days'";
    match sql_query(stmt).execute(conn) {
        Ok(count) => {
            if count > 0 {
                info!("Removed {} old read notifications", count);
            }
        }
        Err(e) => error!("Failed to clean up old notifications: {}", e)
    }
}

/// Update flair aggregate rolling window counts and rankings
fn update_flair_aggregates(conn: &mut PgConnection) {
    info!("Updating flair aggregates...");

    // Update usage_last_day, usage_last_week, usage_last_month from post_flairs
    let update_usage_windows_stmt = r#"
        UPDATE flair_aggregates fa SET
            usage_last_day = COALESCE(pf.day_count, 0) + COALESCE(uf.day_count, 0),
            usage_last_week = COALESCE(pf.week_count, 0) + COALESCE(uf.week_count, 0),
            usage_last_month = COALESCE(pf.month_count, 0) + COALESCE(uf.month_count, 0)
        FROM (
            SELECT flair_template_id,
                COUNT(*) FILTER (WHERE created_at > now() - INTERVAL '1 day') as day_count,
                COUNT(*) FILTER (WHERE created_at > now() - INTERVAL '1 week') as week_count,
                COUNT(*) FILTER (WHERE created_at > now() - INTERVAL '1 month') as month_count
            FROM post_flairs
            GROUP BY flair_template_id
        ) pf
        LEFT JOIN (
            SELECT flair_template_id,
                COUNT(*) FILTER (WHERE created_at > now() - INTERVAL '1 day') as day_count,
                COUNT(*) FILTER (WHERE created_at > now() - INTERVAL '1 week') as week_count,
                COUNT(*) FILTER (WHERE created_at > now() - INTERVAL '1 month') as month_count
            FROM user_flairs
            GROUP BY flair_template_id
        ) uf ON pf.flair_template_id = uf.flair_template_id
        WHERE fa.flair_template_id = pf.flair_template_id
    "#;

    match sql_query(update_usage_windows_stmt).execute(conn) {
        Ok(_) => info!("Updated flair usage windows"),
        Err(e) => error!("Failed to update flair usage windows: {}", e)
    }

    // Update active_user_count (distinct users with this flair currently assigned)
    let update_active_users_stmt = r#"
        UPDATE flair_aggregates fa SET
            active_user_count = COALESCE(uf.active_count, 0)
        FROM (
            SELECT flair_template_id, COUNT(DISTINCT user_id) as active_count
            FROM user_flairs
            GROUP BY flair_template_id
        ) uf
        WHERE fa.flair_template_id = uf.flair_template_id
    "#;

    match sql_query(update_active_users_stmt).execute(conn) {
        Ok(_) => info!("Updated flair active user counts"),
        Err(e) => error!("Failed to update flair active user counts: {}", e)
    }

    // Update avg_post_score, total_post_comments, total_post_score from posts with each flair
    let update_post_stats_stmt = r#"
        UPDATE flair_aggregates fa SET
            avg_post_score = COALESCE(ps.avg_score, 0),
            total_post_comments = COALESCE(ps.total_comments, 0),
            total_post_score = COALESCE(ps.total_score, 0)
        FROM (
            SELECT pf.flair_template_id,
                AVG(pa.score) as avg_score,
                SUM(pa.comments)::int as total_comments,
                SUM(pa.score)::int as total_score
            FROM post_flairs pf
            JOIN post_aggregates pa ON pf.post_id = pa.post_id
            GROUP BY pf.flair_template_id
        ) ps
        WHERE fa.flair_template_id = ps.flair_template_id
    "#;

    match sql_query(update_post_stats_stmt).execute(conn) {
        Ok(_) => info!("Updated flair post stats"),
        Err(e) => error!("Failed to update flair post stats: {}", e)
    }

    // Update trending_score and hot_rank
    // trending_score: weighted recent usage (day*5 + week*2 + month*1)
    // hot_rank: trending_score decayed by age since last use
    let update_rankings_stmt = r#"
        UPDATE flair_aggregates SET
            trending_score = (usage_last_day * 5 + usage_last_week * 2 + usage_last_month),
            hot_rank = CASE
                WHEN last_used_at IS NULL THEN 0
                ELSE (usage_last_day * 5 + usage_last_week * 2 + usage_last_month)::numeric
                    / GREATEST(EXTRACT(EPOCH FROM (now() - last_used_at)) / 3600.0, 1)
            END
    "#;

    match sql_query(update_rankings_stmt).execute(conn) {
        Ok(_) => info!("Updated flair rankings"),
        Err(e) => error!("Failed to update flair rankings: {}", e)
    }

    info!("Done updating flair aggregates.");
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
            "update board_aggregates ba set users_active_{} = mv.count_ from board_aggregates_activity('{}') mv where ba.board_id = mv.board_id",
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
        "UPDATE post_aggregates SET hot_rank = hot_rank(score, created_at)";

    // Update hot_rank_active based on score and newest comment time
    let update_hot_rank_active_stmt =
        "UPDATE post_aggregates SET hot_rank_active = hot_rank(score, newest_comment_time)";

    // Update controversy_rank based on upvotes and downvotes
    let update_controversy_rank_stmt =
        "UPDATE post_aggregates SET controversy_rank = controversy_rank(upvotes, downvotes)";

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
        "UPDATE comment_aggregates SET hot_rank = hot_rank(score, created_at)";

    // Update comment controversy_rank based on upvotes and downvotes
    let update_controversy_rank_stmt =
        "UPDATE comment_aggregates SET controversy_rank = controversy_rank(upvotes, downvotes)";

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
            users = (SELECT COUNT(*) FROM users WHERE deleted_at IS NULL AND is_banned = false),
            posts = (SELECT COUNT(*) FROM posts WHERE deleted_at IS NULL AND is_removed = false),
            comments = (SELECT COUNT(*) FROM comments WHERE deleted_at IS NULL AND is_removed = false),
            boards = (SELECT COUNT(*) FROM boards WHERE deleted_at IS NULL AND is_removed = false),
            upvotes = (SELECT COALESCE(SUM(upvotes), 0) FROM post_aggregates) + (SELECT COALESCE(SUM(upvotes), 0) FROM comment_aggregates),
            downvotes = (SELECT COALESCE(SUM(downvotes), 0) FROM post_aggregates) + (SELECT COALESCE(SUM(downvotes), 0) FROM comment_aggregates)
        WHERE site_id = (SELECT id FROM site LIMIT 1);
    "#;

    match sql_query(update_site_aggregates_stmt).execute(conn) {
        Ok(_) => info!("Updated site aggregates"),
        Err(e) => error!("Failed to update site aggregates: {}", e)
    }

    info!("Done updating site aggregates.");
}

/// Update board aggregates (subscriber count, post count, comment count) every hour
fn update_board_aggregates(conn: &mut PgConnection) {
    info!("Updating board aggregates...");

    // Recalculate board statistics
    let update_board_aggregates_stmt = r#"
        INSERT INTO board_aggregates (board_id, subscribers, posts, comments, created_at)
        SELECT
            b.id as board_id,
            COALESCE(s.subscriber_count, 0) as subscribers,
            COALESCE(p.post_count, 0) as posts,
            COALESCE(c.comment_count, 0) as comments,
            b.created_at
        FROM boards b
        LEFT JOIN (
            SELECT board_id, COUNT(*) as subscriber_count
            FROM board_subscribers
            GROUP BY board_id
        ) s ON b.id = s.board_id
        LEFT JOIN (
            SELECT board_id, COUNT(*) as post_count
            FROM posts
            WHERE deleted_at IS NULL AND is_removed = false
            GROUP BY board_id
        ) p ON b.id = p.board_id
        LEFT JOIN (
            SELECT posts.board_id, COUNT(comments.id) as comment_count
            FROM comments
            JOIN posts ON comments.post_id = posts.id
            WHERE comments.deleted_at IS NULL AND comments.is_removed = false
            GROUP BY posts.board_id
        ) c ON b.id = c.board_id
        ON CONFLICT (board_id) DO UPDATE SET
            subscribers = EXCLUDED.subscribers,
            posts = EXCLUDED.posts,
            comments = EXCLUDED.comments;
    "#;

    match sql_query(update_board_aggregates_stmt).execute(conn) {
        Ok(rows_affected) => info!("Updated board aggregates for {} boards", rows_affected),
        Err(e) => error!("Failed to update board aggregates: {}", e)
    }

    info!("Done updating board aggregates.");
}

/// Update post aggregates (comment counts) on startup
fn update_post_aggregates(conn: &mut PgConnection) {
    info!("Updating post aggregates...");

    let update_post_aggregates_stmt = r#"
        UPDATE post_aggregates pa
        SET comments = COALESCE((
            SELECT COUNT(*)
            FROM comments c
            WHERE c.post_id = pa.post_id AND c.deleted_at IS NULL AND c.is_removed = false
        ), 0)
    "#;

    match sql_query(update_post_aggregates_stmt).execute(conn) {
        Ok(_) => info!("Updated post aggregates"),
        Err(e) => error!("Failed to update post aggregates: {}", e)
    }

    info!("Done updating post aggregates.");
}

/// Update comment aggregates (child counts) on startup
fn update_comment_aggregates(conn: &mut PgConnection) {
    info!("Updating comment aggregates...");

    let update_comment_aggregates_stmt = r#"
        UPDATE comment_aggregates ca
        SET child_count = COALESCE((
            SELECT COUNT(*)::int
            FROM comments c
            WHERE c.parent_id = ca.comment_id AND c.deleted_at IS NULL AND c.is_removed = false
        ), 0)
    "#;

    match sql_query(update_comment_aggregates_stmt).execute(conn) {
        Ok(_) => info!("Updated comment aggregates"),
        Err(e) => error!("Failed to update comment aggregates: {}", e)
    }

    info!("Done updating comment aggregates.");
}

/// Pre-create next month's partition for the notifications table.
/// Runs daily so partitions are always created well ahead of time.
/// Uses CREATE TABLE IF NOT EXISTS to be safe for repeated execution.
fn ensure_partitions(conn: &mut PgConnection) {
    info!("Checking partitions for next month...");

    // The only partitioned table is `notifications`, with naming convention
    // `notifications_YYYY_MM` (e.g., notifications_2026_04).
    let partitioned_tables = &["notifications"];

    let today = chrono::Utc::now().date_naive();

    // Create partitions for the next 2 months to provide extra safety margin
    for months_ahead in 1..=2 {
        let target = add_months(today, months_ahead);
        let year = target.year();
        let month = target.month();

        // Compute the range boundaries: first day of target month to first day of next month
        let range_start = format!("{:04}-{:02}-01", year, month);
        let next_month = add_months(target, 1);
        let range_end = format!("{:04}-{:02}-01", next_month.year(), next_month.month());

        for table in partitioned_tables {
            let partition_name = format!("{}_{:04}_{:02}", table, year, month);

            // Check if partition already exists by querying pg_class
            let check_query = format!(
                "SELECT 1 FROM pg_class WHERE relname = '{}'",
                partition_name
            );
            match sql_query(&check_query).execute(conn) {
                Ok(count) if count > 0 => {
                    // Partition already exists
                    continue;
                }
                Ok(_) => {
                    // Partition does not exist — create it
                    let create_query = format!(
                        "CREATE TABLE IF NOT EXISTS {} PARTITION OF {} FOR VALUES FROM ('{}') TO ('{}')",
                        partition_name, table, range_start, range_end
                    );
                    match sql_query(&create_query).execute(conn) {
                        Ok(_) => info!("Created partition: {}", partition_name),
                        Err(e) => error!("Failed to create partition {}: {}", partition_name, e),
                    }
                }
                Err(e) => {
                    error!("Failed to check if partition {} exists: {}", partition_name, e);
                }
            }
        }
    }

    info!("Done checking partitions.");
}

/// Add N months to a NaiveDate, handling year rollovers.
fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total_months = date.month0() + months;
    let new_year = date.year() + (total_months / 12) as i32;
    let new_month = (total_months % 12) + 1;
    NaiveDate::from_ymd_opt(new_year, new_month, 1).unwrap_or(date)
}
