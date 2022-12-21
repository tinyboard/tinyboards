use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::PurgeUser,
    data::TinyBoardsContext,
    moderator::ModActionResponse,
    request::purge_image_from_pictrs,
    utils::{blocking, purge_image_posts_for_user, require_user},
};
use tinyboards_db::{
    models::{
        moderator::admin_actions::{AdminPurgeUser, AdminPurgeUserForm},
        user::users::User,
    },
    traits::Crud,
};
use tinyboards_utils::{error::TinyBoardsError, settings::SETTINGS};
use url::Url;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for PurgeUser {
    type Response = ModActionResponse<AdminPurgeUser>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &PurgeUser = &self;
        let target_user_id = data.user_id;

        let settings = SETTINGS.to_owned();

        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let target_user = blocking(context.pool(), move |conn| {
            User::read(conn, target_user_id.clone())
        })
        .await??;

        if target_user
            .name
            .to_lowercase()
            .contains(settings.owner_name.to_lowercase().as_str())
            || target_user.is_admin
        {
            return Err(TinyBoardsError::from_message(
                "can't purge a site admin or the owner",
            ));
        }

        // purge user banner
        if let Some(banner) = target_user.banner {
            purge_image_from_pictrs(
                context.client(),
                context.settings(),
                &Url::parse(banner.as_str()).unwrap(),
            )
            .await
            .ok();
        }

        // purge user avatar
        if let Some(avatar) = target_user.avatar {
            purge_image_from_pictrs(
                context.client(),
                context.settings(),
                &Url::parse(avatar.as_str()).unwrap(),
            )
            .await
            .ok();
        }

        // purge all submitted media/images from user
        purge_image_posts_for_user(
            target_user_id,
            context.pool(),
            context.settings(),
            context.client(),
        )
        .await?;

        // delete user
        blocking(context.pool(), move |conn| {
            User::delete(conn, target_user_id)
        })
        .await??;

        let reason = data.reason.clone();

        let form = AdminPurgeUserForm {
            admin_id: user.id,
            user_id: target_user_id.clone(),
            reason: Some(reason),
        };

        let mod_action = blocking(context.pool(), move |conn| {
            AdminPurgeUser::create(conn, &form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}
