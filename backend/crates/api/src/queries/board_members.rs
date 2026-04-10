use async_graphql::*;
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Uuid as DieselUuid};
use diesel_async::RunQueryDsl;
use std::collections::HashMap;
use tinyboards_db::{
    models::{
        aggregates::UserAggregates as DbUserAggregates,
        user::user::User as DbUser,
    },
    schema::{users, wiki_page_revisions, wiki_pages},
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::{helpers::permissions, structs::user::User as GqlUser};

#[derive(Default)]
pub struct QueryBoardMembers;

/// A user with their contribution score for the board.
#[derive(SimpleObject)]
pub struct BoardContributor {
    pub user: GqlUser,
    pub post_score: i64,
    pub comment_score: i64,
    pub total_score: i64,
}

/// A user who has edited wiki pages in a board.
#[derive(SimpleObject)]
pub struct WikiContributor {
    pub user: GqlUser,
    pub edit_count: i64,
}

#[Object]
impl QueryBoardMembers {
    /// Top contributors to a board over the last 30 days, ranked by combined
    /// post + comment score. Public query.
    pub async fn get_top_contributors(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        limit: Option<i64>,
    ) -> Result<Vec<BoardContributor>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let _viewer = permissions::optional_auth(ctx);

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let limit = limit.unwrap_or(10).min(25);
        let cutoff = Utc::now() - chrono::Duration::days(30);

        // Aggregate post scores per creator in this board over the last 30 days.
        #[derive(QueryableByName)]
        struct ScoreRow {
            #[diesel(sql_type = DieselUuid)]
            creator_id: Uuid,
            #[diesel(sql_type = BigInt)]
            total: i64,
        }

        let post_scores: Vec<ScoreRow> = diesel::sql_query(
            "SELECT creator_id, COALESCE(SUM(score), 0)::bigint AS total \
             FROM post_aggregates \
             WHERE board_id = $1 AND created_at >= $2 \
             GROUP BY creator_id",
        )
        .bind::<DieselUuid, _>(board_uuid)
        .bind::<diesel::sql_types::Timestamptz, _>(cutoff)
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let comment_scores: Vec<ScoreRow> = diesel::sql_query(
            "SELECT c.creator_id, COALESCE(SUM(ca.score), 0)::bigint AS total \
             FROM comments c \
             INNER JOIN comment_aggregates ca ON ca.comment_id = c.id \
             WHERE c.board_id = $1 AND c.created_at >= $2 \
               AND c.deleted_at IS NULL AND c.is_removed = false \
             GROUP BY c.creator_id",
        )
        .bind::<DieselUuid, _>(board_uuid)
        .bind::<diesel::sql_types::Timestamptz, _>(cutoff)
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Merge into a single map: user_id -> (post_score, comment_score).
        let mut scores: HashMap<Uuid, (i64, i64)> = HashMap::new();
        for row in &post_scores {
            scores.entry(row.creator_id).or_insert((0, 0)).0 = row.total;
        }
        for row in &comment_scores {
            scores.entry(row.creator_id).or_insert((0, 0)).1 = row.total;
        }

        // Sort by total score descending, take top N.
        let mut ranked: Vec<(Uuid, i64, i64)> = scores
            .into_iter()
            .map(|(uid, (ps, cs))| (uid, ps, cs))
            .collect();
        ranked.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2)));
        ranked.truncate(limit as usize);

        if ranked.is_empty() {
            return Ok(vec![]);
        }

        // Load user profiles and aggregates.
        let user_ids: Vec<Uuid> = ranked.iter().map(|(uid, _, _)| *uid).collect();

        let db_users: Vec<DbUser> = users::table
            .filter(users::id.eq_any(&user_ids))
            .filter(users::deleted_at.is_null())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let aggs: Vec<DbUserAggregates> = tinyboards_db::schema::user_aggregates::table
            .filter(tinyboards_db::schema::user_aggregates::user_id.eq_any(&user_ids))
            .load(conn)
            .await
            .unwrap_or_default();

        // Build output in ranked order.
        let contributors = ranked
            .into_iter()
            .filter_map(|(uid, ps, cs)| {
                let user_db = db_users.iter().find(|u| u.id == uid)?;
                let agg = aggs.iter().find(|a| a.user_id == uid).cloned();
                Some(BoardContributor {
                    user: GqlUser::from_db(user_db.clone(), agg),
                    post_score: ps,
                    comment_score: cs,
                    total_score: ps + cs,
                })
            })
            .collect();

        Ok(contributors)
    }

    /// Users who have contributed to wiki pages in a board, ranked by number
    /// of edits. Only returns results if the board has wiki enabled. Public query.
    pub async fn get_wiki_contributors(
        &self,
        ctx: &Context<'_>,
        board_id: ID,
        limit: Option<i64>,
    ) -> Result<Vec<WikiContributor>> {
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let _viewer = permissions::optional_auth(ctx);

        let board_uuid: Uuid = board_id
            .parse()
            .map_err(|_| TinyBoardsError::BadRequest("Invalid board ID".to_string()))?;

        let limit = limit.unwrap_or(10).min(25);

        // Count edits per editor across all wiki pages for this board.
        let edit_counts: Vec<(Uuid, i64)> = wiki_page_revisions::table
            .inner_join(
                wiki_pages::table
                    .on(wiki_pages::id.eq(wiki_page_revisions::page_id)),
            )
            .filter(wiki_pages::board_id.eq(board_uuid))
            .filter(wiki_pages::deleted_at.is_null())
            .group_by(wiki_page_revisions::editor_id)
            .select((
                wiki_page_revisions::editor_id,
                diesel::dsl::count(wiki_page_revisions::id),
            ))
            .order(diesel::dsl::count(wiki_page_revisions::id).desc())
            .limit(limit)
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        if edit_counts.is_empty() {
            return Ok(vec![]);
        }

        let user_ids: Vec<Uuid> = edit_counts.iter().map(|(uid, _)| *uid).collect();

        let db_users: Vec<DbUser> = users::table
            .filter(users::id.eq_any(&user_ids))
            .filter(users::deleted_at.is_null())
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let aggs: Vec<DbUserAggregates> = tinyboards_db::schema::user_aggregates::table
            .filter(tinyboards_db::schema::user_aggregates::user_id.eq_any(&user_ids))
            .load(conn)
            .await
            .unwrap_or_default();

        let contributors = edit_counts
            .into_iter()
            .filter_map(|(uid, count)| {
                let user_db = db_users.iter().find(|u| u.id == uid)?;
                let agg = aggs.iter().find(|a| a.user_id == uid).cloned();
                Some(WikiContributor {
                    user: GqlUser::from_db(user_db.clone(), agg),
                    edit_count: count,
                })
            })
            .collect();

        Ok(contributors)
    }
}
