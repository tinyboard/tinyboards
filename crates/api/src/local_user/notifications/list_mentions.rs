use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{GetPersonMentions, GetPersonMentionsResponse},
    utils::{get_local_user_view_from_jwt},
};
use tinyboards_db::{
    map_to_comment_sort_type,
    CommentSortType
};
use tinyboards_db_views::person_mention_view::PersonMentionQuery;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetPersonMentions {
    type Response = GetPersonMentionsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<GetPersonMentionsResponse, TinyBoardsError> {
            let data: &GetPersonMentions = &self;
            
            let person =
            get_local_user_view_from_jwt(auth, context.pool(), context.master_key())
            .await?
            .person;
        
            let sort = match data.sort.as_ref() {
                Some(sort) => map_to_comment_sort_type(Some(&sort.to_lowercase())),
                None => CommentSortType::Hot,
            };
            let page = data.page;
            let limit = data.limit;
            let unread_only = data.unread_only;
            let person_id = Some(person.id);
            
            let resp = PersonMentionQuery::builder()
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

            Ok(GetPersonMentionsResponse { mentions: resp.mentions, total_count: resp.count, unread_count: resp.unread })
    }
}