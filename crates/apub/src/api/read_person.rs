use crate::{api::PerformApub, fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{GetPersonDetails, GetPersonDetailsResponse},
    utils::{check_private_instance, is_admin, load_user_opt},
};
use tinyboards_db::{
    map_to_sort_type,
    models::person::{local_user::AdminPerms, person::Person},
    utils::post_to_comment_sort_type,
};
use tinyboards_db_views::{
    comment_view::{CommentQuery, CommentQueryResponse},
    post_view::{PostQuery, PostQueryResponse},
    structs::{BoardModeratorView, PersonView},
};
use tinyboards_federation::config::Data;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait]
impl PerformApub for GetPersonDetails {
    type Response = GetPersonDetailsResponse;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        &self,
        context: &Data<TinyBoardsContext>,
        auth: Option<&str>,
    ) -> Result<GetPersonDetailsResponse, TinyBoardsError> {
        let data: &GetPersonDetails = self;

        // check to make sure a person name or id is given
        if data.username.is_none() && data.person_id.is_none() {
            return Err(TinyBoardsError::from_message(400, "no id provided."));
        }

        let view = load_user_opt(context.pool(), context.master_key(), auth).await?;
        let _is_admin = view.as_ref().map(|luv| is_admin(luv).is_ok());

        check_private_instance(&view, context.pool()).await?;

        let person_details_id = match data.person_id {
            Some(id) => id,
            None => {
                if let Some(username) = &data.username {
                    resolve_actor_identifier::<ApubPerson, Person>(username, context, &view, true)
                        .await?
                        .id
                } else {
                    return Err(TinyBoardsError::from_message(
                        400,
                        "couldn't find that username or email.",
                    ));
                }
            }
        };

        // no need to return settings for the user, since this comes back with GetSite
        let person_view = PersonView::read(context.pool(), person_details_id, false).await?;

        let sort = data.sort.clone().map(|x| x.to_lowercase());
        let sort = Some(map_to_sort_type(match sort {
            Some(ref sort) => sort,
            None => "hot",
        }));

        let show_deleted = match view {
            Some(ref v) => v.local_user.has_permission(AdminPerms::Users),
            None => false,
        };

        let show_removed = match view {
            Some(ref v) => {
                v.local_user.has_permission(AdminPerms::Users)
                    || v.person.id == person_view.person.id
            }
            None => false,
        };

        let page = data.page;
        let limit = data.limit;
        let saved_only = data.saved_only;
        let board_id = data.board_id;
        let local_user = view.map(|l| l.local_user);
        let local_user_clone = local_user.clone();

        let posts_query = PostQuery::builder()
            .pool(context.pool())
            .sort(sort)
            .saved_only(saved_only)
            .user(local_user.as_ref())
            .show_deleted(Some(show_deleted))
            .show_removed(Some(show_removed))
            .board_id(board_id)
            .page(page)
            .limit(limit);

        // If its saved only, you don't care what creator it was.
        // Or, if it is not saved, then you only want it for a specific creator
        let PostQueryResponse {
            posts,
            count: posts_count_total,
        } = if !saved_only.unwrap_or(false) {
            posts_query
                .creator_id(Some(person_details_id))
                .build()
                .list()
        } else {
            posts_query.build().list()
        }
        .await?;

        let comments_query = CommentQuery::builder()
            .pool(context.pool())
            .user(local_user_clone.as_ref())
            .sort(sort.map(post_to_comment_sort_type))
            .saved_only(saved_only)
            .show_deleted(Some(show_deleted))
            .show_removed(Some(show_removed))
            .board_id(board_id)
            .page(page)
            .limit(limit);

        // If its saved only, you don't care what creator it was
        // Or, if its not saved, then you only want it for that specific creator
        let CommentQueryResponse {
            comments,
            count: comments_count_total,
        } = if !saved_only.unwrap_or(false) {
            comments_query
                .creator_id(Some(person_details_id))
                .build()
                .list()
        } else {
            comments_query.build().list()
        }
        .await?;

        let moderates = BoardModeratorView::for_user(context.pool(), person_details_id).await?;

        Ok(GetPersonDetailsResponse {
            person_view,
            comments,
            posts,
            comments_count_total,
            posts_count_total,
            moderates,
        })
    }
}
