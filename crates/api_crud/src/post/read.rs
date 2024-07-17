use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{GetPost, GetPostResponse, PostIdPath},
    utils::{check_private_instance, is_mod_or_admin, load_user_opt},
};

use tinyboards_db::{
    models::{
        board::board_mods::{BoardModerator, ModPerms},
        person::local_user::AdminPerms,
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_utils::TinyBoardsError;

use tinyboards_db_views::{
    structs::{BoardModeratorView, BoardView, LocalUserView, PersonView, PostView},
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

        let v = load_user_opt(context.pool(), context.master_key(), auth).await?;

        // check to see if instance is set to private before listing post
        check_private_instance(&v, context.pool()).await?;

        let person_id = match v {
            Some(ref v) => Some(v.person.id),
            None => None,
        };

        let post_id = path.post_id;
        let post = Post::read(context.pool(), post_id).await?;
        /*let mut mod_or_admin = false;
        if let Some(pid) = person_id {
            mod_or_admin = is_mod_or_admin(context.pool(), pid, post.board_id).await.is_ok();
            }*/

        let is_admin = match v {
            Some(LocalUserView { ref local_user, .. }) => {
                local_user.has_permission(AdminPerms::Content)
            }
            None => false,
        };

        let is_mod = match person_id {
            Some(person_id) => {
                let mod_rel = BoardModerator::get_by_person_id_for_board(
                    context.pool(),
                    person_id,
                    post.board_id,
                    true,
                )
                .await;

                match mod_rel {
                    Ok(m) => m.has_permission(ModPerms::Content),
                    Err(_) => false,
                }
            }
            None => false,
        };

        let mod_or_admin = is_admin || is_mod;

        let mut post_view =
            PostView::read(context.pool(), post_id, person_id, Some(mod_or_admin)).await?;

        let author = PersonView::read(context.pool(), post_view.post.creator_id, false).await?;

        if post_view.post.is_removed || post_view.post.is_deleted {
            post_view.hide_if_removed_or_deleted(
                v.as_ref().map(|view| view.person.id),
                is_admin,
                is_mod,
            );
        }

        let _post_id = post_view.post.id;

        // mark read here

        let board_id = post_view.board.id;
        let board_view =
            BoardView::read(context.pool(), board_id, person_id, Some(mod_or_admin)).await?;

        // blank out deleted or removed info here

        let moderators = BoardModeratorView::for_board(context.pool(), board_id).await?;

        let author_counts = author.counts;

        Ok(GetPostResponse {
            post_view,
            board_view,
            moderators,
            author_counts,
        })
    }
}
