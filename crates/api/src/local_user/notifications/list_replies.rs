use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{GetCommentReplies, GetCommentRepliesResponse},
    utils::require_user,
};
use tinyboards_db::{
    map_to_comment_sort_type,
    CommentSortType
};
use tinyboards_db_views::{comment_reply_view::CommentReplyQuery, structs::CommentReplyView};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetCommentReplies {
    type Response = GetCommentRepliesResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<GetCommentRepliesResponse, TinyBoardsError> {
            let data: &GetCommentReplies = &self;

            let person =
            require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?
            .person;
        
            let sort = match data.sort.as_ref() {
                Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
                None => CommentSortType::Hot,
            };
            let page = data.page;
            let limit = data.limit;
            let unread_only = data.unread_only;
            let person_id = Some(person.id);
            
            let resp = CommentReplyQuery::builder()
                .pool(context.pool())
                .recipient_id(person_id)
                .person_id(person_id)
                .sort(Some(sort))
                .unread_only(unread_only)
                .page(page)
                .limit(limit)
                .build()
                .list()
                .await?;

            // mark all replies as read when visiting the inbox
            CommentReplyView::mark_all_replies_as_read(context.pool(), person.id.clone()).await?;

            Ok(GetCommentRepliesResponse { replies: resp.replies, total_count: resp.count, unread_count: resp.unread })
    }
}