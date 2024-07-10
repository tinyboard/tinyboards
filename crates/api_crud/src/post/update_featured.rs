use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{PostIdPath, PostResponse, TogglePostFeatured},
    utils::{is_mod_or_admin, require_user},
};
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::{
        moderator::mod_actions::{ModFeaturePost, ModFeaturePostForm},
        person::local_user::AdminPerms,
        post::posts::Post,
    },
    traits::Crud,
    PostFeatureType,
};
use tinyboards_db_views::structs::PostView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for TogglePostFeatured {
    type Response = PostResponse;
    type Route = PostIdPath;

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        PostIdPath { post_id }: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &TogglePostFeatured = &self;

        //let post_id = data.post_id;
        let featured = data.value;
        let feature_type = data.feature_type;

        // get the post object
        let post = Post::read(context.pool(), post_id).await?;

        // you need to at least be a board moderator for this action, if the feature type is local then you need admin
        let mut view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), post.board_id, ModPerms::Content, None)
            .await
            .unwrap()?;

        if feature_type == PostFeatureType::Local {
            // if it's a local site feature you need admin perms to feature (sticky)
            view = require_user(context.pool(), context.master_key(), auth)
                .await
                .require_admin(AdminPerms::Content)
                .unwrap()?;
        }

        if feature_type == PostFeatureType::Board {
            post.set_featured_board(context.pool(), featured).await?;
        } else {
            post.set_featured_local(context.pool(), featured).await?;
        }

        // form for submitting post sticky action to the mod log
        let feature_post_form = ModFeaturePostForm {
            mod_person_id: view.person.id,
            post_id: post_id,
            featured: Some(featured),
        };

        // submit mod action to the mod log
        ModFeaturePost::create(context.pool(), &feature_post_form).await?;

        let is_mod_or_admin = is_mod_or_admin(context.pool(), view.person.id, post.board_id)
            .await
            .is_ok();

        // get post view
        let post_view =
            PostView::read(context.pool(), post.id, None, Some(is_mod_or_admin)).await?;

        Ok(PostResponse { post_view })
    }
}
