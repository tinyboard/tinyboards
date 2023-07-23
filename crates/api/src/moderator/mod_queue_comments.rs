use tinyboards_api_common::utils::require_user;
use actix_web::web::Data;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_utils::TinyBoardsError;
use tinyboards_db_views::comment_mod_queue_view::{CommentModQuery, CommentModQueryResponse};
use tinyboards_api_common::{comment::GetCommentsResponse, moderator::CommentModQueue};
use crate::Perform;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for CommentModQueue {
	type Response = GetCommentsResponse;
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
    		.require_board_mod(self.board_id.unwrap_or(1), context.pool())
    		.await
    		.unwrap()?;

    	let CommentModQueryResponse { comments, count } = CommentModQuery::builder()
    		.pool(context.pool())
    		.my_person_id(v.person.id)
    		.page(data.page)
    		.limit(data.limit)
    		// .board_id(data.board_id)
    		.build()
    		.list()
    		.await
    		.map_err(|e| TinyBoardsError::from_error_message(e, 500, "Couldn't load reported comments."))?;

    	Ok(GetCommentsResponse { comments, total_count: count })
    }
}
