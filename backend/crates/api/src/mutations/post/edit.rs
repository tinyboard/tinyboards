use crate::helpers::permissions;
use crate::structs::post::Post;
use crate::DbPool;
use crate::Settings;
use crate::utils::emoji::process_content_with_emojis;
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::{
        aggregates::PostAggregates,
        post::posts::{Post as DbPost, PostUpdateForm},
        site::site::Site,
    },
    schema::{boards, post_aggregates, posts, site},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[derive(Default)]
pub struct EditPost;

#[Object]
impl EditPost {
    pub async fn edit_post(
        &self,
        ctx: &Context<'_>,
        id: ID,
        body: String,
        alt_text: Option<String>,
    ) -> Result<Post> {
        let user = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();
        let conn = &mut get_conn(pool).await?;

        let post_uuid: Uuid = id
            .parse()
            .map_err(|_| TinyBoardsError::from_message(400, "Invalid post ID"))?;

        let post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;

        if user.id != post.creator_id {
            return Err(
                TinyBoardsError::from_message(403, "You can only edit your own posts").into(),
            );
        }

        if post.deleted_at.is_some() || post.is_removed {
            return Err(TinyBoardsError::from_message(
                404,
                "Your post has been deleted or removed.",
            )
            .into());
        }

        // Check board is not banned
        let board: tinyboards_db::models::board::boards::Board = boards::table
            .find(post.board_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Board not found".into()))?;

        if board.is_banned {
            let reason = board
                .public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        // Load site config for emoji settings
        let site_config: Site = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let emoji_limit = if site_config.emoji_enabled {
            site_config
                .max_emojis_per_post
                .map(|limit| limit as usize)
        } else {
            Some(0)
        };

        let body_html = Some(
            process_content_with_emojis(&body, pool, Some(board.id), settings, emoji_limit)
                .await?,
        );

        // Clean up orphaned uploads
        let storage = ctx.data::<crate::storage::StorageBackend>()?;
        if let Some(ref html) = body_html {
            if let Err(e) = crate::helpers::files::cleanup::cleanup_orphaned_uploads(
                pool,
                post_uuid,
                true,
                html,
                storage,
            )
            .await
            {
                tracing::error!("Failed to cleanup orphaned uploads: {:?}", e);
            }
        }

        let form = PostUpdateForm {
            body: Some(body),
            body_html,
            updated_at: Some(chrono::Utc::now()),
            alt_text: Some(alt_text),
            ..PostUpdateForm::default()
        };

        diesel::update(posts::table.find(post_uuid))
            .set(&form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let db_post: DbPost = posts::table
            .find(post_uuid)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found".into()))?;
        let agg: PostAggregates = post_aggregates::table
            .filter(post_aggregates::post_id.eq(post_uuid))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post aggregates not found".into()))?;

        Ok(Post::from((db_post, agg)))
    }
}
