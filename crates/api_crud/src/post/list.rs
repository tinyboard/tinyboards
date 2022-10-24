use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    post::{ListPosts, ListPostsResponse},
    utils::{blocking, get_user_view_from_jwt_opt, check_private_instance},
};
use porpl_db::{
    map_to_sort_type,
    map_to_listing_type, traits::DeleteableOrRemoveable,
};
use porpl_db_views::post_view::PostQuery;
use porpl_utils::error::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for ListPosts {
    type Response = ListPostsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<ListPostsResponse, PorplError> {
        let data: ListPosts = self;

        // check to see if user is logged in or not
        let user_view =
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;
        
        // check to see if the instance is private or not before listing
        check_private_instance(
            &user_view, 
            context.pool()
        ).await?;

        let is_logged_in = user_view.is_some();
        
        let user = user_view.map(|u| u.user);
        let sort = map_to_sort_type(data.sort.as_deref());
        let listing_type = map_to_listing_type(data.listing_type.as_deref());
        let page = data.page;
        let limit = data.limit;
        let board_id = data.board_id;
        let saved_only = data.saved_only;

        let mut posts = blocking(context.pool(), move |conn| {
            PostQuery::builder()
                .conn(conn)
                .listing_type(Some(listing_type))
                .sort(Some(sort))
                .board_id(board_id)
                .user(user)
                .saved_only(saved_only)
                .page(page)
                .limit(limit)
                .build()
                .list()
        })
        .await?
        .map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::err_500()
        })?;

        if !is_logged_in {
            for pv in posts
                .iter_mut()
                .filter(|p| p.post.deleted || p.post.removed)
            {
                pv.post = pv.to_owned().post.blank_out_deleted_info();
            }

            for pv in posts
                .iter_mut()
                .filter(|p| p.board.deleted)
            {
                pv.board = pv.to_owned().board.blank_out_deleted_info();
            }
        }

        Ok(ListPostsResponse { posts })
    }
}
