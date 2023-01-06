use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{GetUserMentions, GetUserMentionsResponse},
    utils::{require_user, blocking},
};
use tinyboards_db::{
    map_to_comment_sort_type,
    CommentSortType
};
use tinyboards_db_views::user_mention_view::UserMentionQuery;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetUserMentions {
    type Response = GetUserMentionsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<GetUserMentionsResponse, TinyBoardsError> {
        let data: &GetUserMentions = &self;

        println!("here");

        let user = 
            require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;
        
            let sort = match data.sort.as_ref() {
                Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
                None => CommentSortType::Hot,
            };
            let page = data.page;
            let limit = data.limit;
            let unread_only = data.unread_only;
            let user_id = Some(user.id);

            println!("user_id: {:?}", user_id);
            
            let mentions = blocking(context.pool(), move |conn| {
                UserMentionQuery::builder()
                    .conn(conn)
                    .recipient_id(user_id)
                    .user_id(user_id)
                    .sort(Some(sort))
                    .unread_only(unread_only)
                    .page(page)
                    .limit(limit)
                    .build()
                    .list()
            })
            .await??;

            Ok(GetUserMentionsResponse { mentions })
    }
}