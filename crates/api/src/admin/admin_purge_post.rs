use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::{PurgeItemResponse, PurgePost},
    data::TinyBoardsContext,
    utils::{purge_local_image_by_url, require_user},
};
use tinyboards_db::{
    models::{
        moderator::admin_actions::{AdminPurgePost, AdminPurgePostForm},
        person::local_user::AdminPerms,
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PurgePost {
    type Response = PurgeItemResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &PurgePost = &self;

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Content)
            .unwrap()?;

        let target_post_id = data.post_id;
        let reason = data.reason.clone();

        let post = Post::read(context.pool(), target_post_id).await?;

        // purge image
        if let Some(url) = post.url {
            purge_local_image_by_url(context.pool(), &url).await.ok();
        }

        // purge thumbnail
        if let Some(thumbnail_url) = post.thumbnail_url {
            purge_local_image_by_url(context.pool(), &thumbnail_url)
                .await
                .ok();
        }

        // delete post
        Post::delete(context.pool(), target_post_id).await?;

        let form = AdminPurgePostForm {
            admin_id: view.person.id,
            post_id: target_post_id,
            reason: Some(reason),
        };

        // submit mod log action
        AdminPurgePost::create(context.pool(), &form).await?;

        Ok(PurgeItemResponse { success: true })
    }
}
