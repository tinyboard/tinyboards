use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::PurgePost,
    utils::{blocking, require_user}, 
    moderator::ModActionResponse, 
    data::TinyBoardsContext, 
    request::purge_image_from_pictrs,
};
use tinyboards_db::{
    models::{
        moderator::admin_actions::{AdminPurgePost, AdminPurgePostForm},
        post::post::Post,
    },
    traits::Crud,
};
use tinyboards_utils::{
    error::TinyBoardsError,
};
use url::Url;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PurgePost {
    type Response = ModActionResponse<AdminPurgePost>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &PurgePost = &self;

        let user
            = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let target_post_id = data.post_id;
        let reason = data.reason.clone();

        let post = blocking(context.pool(), move |conn| {
            Post::read(conn, target_post_id)
        })
        .await??;

        // purge image
        if let Some(url) = post.url {
            let parsed_url = Url::parse(url.as_str()).unwrap();
            purge_image_from_pictrs(context.client(), context.settings(), &parsed_url)
            .await
            .ok();
        }

        // purge thumbnail
        if let Some(url) = post.thumbnail_url {
            let parsed_url = Url::parse(url.as_str()).unwrap();
            purge_image_from_pictrs(context.client(), context.settings(), &parsed_url)
            .await
            .ok();
        }

        // delete post
        blocking(context.pool(), move |conn| {
            Post::delete(conn, target_post_id)
        })
        .await??;

        let form = AdminPurgePostForm {
            admin_id: user.id,
            post_id: target_post_id,
            reason: Some(reason),
        };

        // submit mod log action
        let mod_action = blocking(context.pool(), move |conn| {
            AdminPurgePost::create(conn, &form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}