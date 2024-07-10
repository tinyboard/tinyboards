use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::utils::require_user;
use tinyboards_api_common::{moderator::PostModQueue, post::GetPostsResponse};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db_views::post_mod_queue_view::PostModQuery;
use tinyboards_db_views::post_mod_queue_view::PostModQueryResponse;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PostModQueue {
    type Response = GetPostsResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &Self = &self;

        let v = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(
                context.pool(),
                self.board_id.unwrap_or(1),
                ModPerms::Content,
                None,
            )
            .await
            .unwrap()?;

        let PostModQueryResponse { posts, count } = PostModQuery::builder()
            .pool(context.pool())
            .my_person_id(v.person.id)
            .admin(v.person.is_admin)
            .page(data.page)
            .limit(data.limit)
            .board_id(data.board_id)
            .build()
            .list()
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Couldn't load reported posts.")
            })?;

        Ok(GetPostsResponse {
            posts,
            total_count: count,
        })
    }
}
