use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{GetPost, GetPostResponse, PostIdPath},
    utils::{check_private_instance, load_local_user_opt, is_mod_or_admin},
};

use tinyboards_db::{models::post::posts::Post, traits::Crud};
use tinyboards_utils::TinyBoardsError;

use tinyboards_db_views::{
    structs::{BoardModeratorView, BoardView, PostView, PersonView},
    DeleteableOrRemoveable,
};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for GetPost {
    type Response = GetPostResponse;
    type Route = PostIdPath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<GetPostResponse, TinyBoardsError> {
        let _data = self;

        let local_user = load_local_user_opt(context.pool(), context.master_key(), auth).await?;

        // check to see if instance is set to private before listing post
        check_private_instance(&local_user, context.pool()).await?;

        let person_id = match local_user {
            Some(ref local_user) => Some(local_user.id),
            None => None,
        };

        let post_id = path.post_id;
        let post = Post::read(context.pool(), post_id).await?;
        let mut mod_or_admin = false;
        if let Some(pid) = person_id {
            mod_or_admin = is_mod_or_admin(context.pool(), pid, post.board_id).await.is_ok();
        } 
        

        let mut post_view = PostView::read(context.pool(), post_id, person_id, Some(mod_or_admin)).await?;

        if post_view.post.is_removed || post_view.post.is_deleted {
            post_view.hide_if_removed_or_deleted(local_user.as_ref());
        }

        let _post_id = post_view.post.id;

        // mark read here

        let board_id = post_view.board.id;
        let board_view = BoardView::read(context.pool(), board_id, person_id, Some(mod_or_admin)).await?;

        // blank out deleted or removed info here

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;
        
        let author = PersonView::read(context.pool(), post_view.post.creator_id).await?;

        let author_counts = author.counts;
        
        Ok(GetPostResponse {
            post_view,
            board_view,
            moderators,
            author_counts,
        })
    }
}
