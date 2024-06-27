use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    post::{FeaturePost, PostResponse},
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
impl<'des> Perform<'des> for FeaturePost {
    type Response = PostResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &FeaturePost = &self;

        let post_id = data.post_id;
        let featured = data.featured;
        let feature_type = data.feature_type;

        // get the post object
        let orig_post = Post::read(context.pool(), post_id.clone()).await?;

        // you need to at least be a board moderator for this action, if the feature type is local then you need admin
        let mut view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_board_mod(context.pool(), orig_post.board_id, ModPerms::Content)
            .await
            .unwrap()?;

        if feature_type == PostFeatureType::Local {
            // if it's a local site feature you need admin perms to feature (sticky)
            view = require_user(context.pool(), context.master_key(), auth)
                .await
                .require_admin(AdminPerms::Content)
                .unwrap()?;
        }

        let updated_post = if feature_type == PostFeatureType::Board {
            Post::update_featured_board(context.pool(), post_id, data.featured).await?
        } else {
            Post::update_featured_local(context.pool(), post_id, data.featured).await?
        };

        // form for submitting post sticky action to the mod log
        let feature_post_form = ModFeaturePostForm {
            mod_person_id: view.person.id,
            post_id: post_id.clone(),
            featured: Some(featured.clone()),
        };

        // submit mod action to the mod log
        ModFeaturePost::create(context.pool(), &feature_post_form).await?;

        let is_mod_or_admin =
            is_mod_or_admin(context.pool(), view.person.id, updated_post.board_id)
                .await
                .is_ok();

        // get post view
        let post_view =
            PostView::read(context.pool(), updated_post.id, None, Some(is_mod_or_admin)).await?;

        Ok(PostResponse { post_view })
    }
}
