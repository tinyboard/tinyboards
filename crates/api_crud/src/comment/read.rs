use crate::PerformCrud;
use actix_web::web;
use porpl_api_common::{
    comment::{GetComment, CommentResponse, CommentIdPath},
    data::PorplContext,
    utils::{blocking, get_user_view_from_jwt_opt, check_private_instance},
};
use porpl_db_views::structs::CommentView;
use porpl_utils::PorplError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;

    async fn perform(
        self,
        context: &web::Data<PorplContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, PorplError> {
        
        let _data = self;

        let user_view =
            get_user_view_from_jwt_opt(auth, context.pool(), context.master_key()).await?;
        
        // check if the instance is private before listing comments
        check_private_instance(
            &user_view, 
            context.pool()
        )
        .await?;

        let user_id = user_view.map(|u| u.user.id);
        let comment_id = path.comment_id;

        let comment_view = blocking(context.pool(), move |conn| {
            CommentView::read(conn, comment_id, user_id)
                .map_err(|_e| PorplError::from_string("could not find comment", 404))
        })
        .await??;
        
        Ok( CommentResponse { comment_view } )
    }
}
