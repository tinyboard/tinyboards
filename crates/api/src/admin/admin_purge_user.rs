use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::PurgeUser,
    data::TinyBoardsContext,
    moderator::ModActionResponse,
    utils::{purge_local_image_posts_for_user, require_user, purge_local_image_by_url},
};
use tinyboards_db::{
    models::{
        moderator::admin_actions::{AdminPurgeUser, AdminPurgeUserForm},
        person::local_user::LocalUser,
        person::person::Person,
    },
    traits::Crud,
};
use tinyboards_utils::{error::TinyBoardsError, settings::SETTINGS};

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
        let target_person_id = data.person_id;

        let settings = SETTINGS.to_owned();

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let target_user = LocalUser::get_by_person_id(context.pool(), target_person_id.clone()).await?;
        let target_person = Person::read(context.pool(), target_person_id.clone()).await?;

        if target_user
            .name
            .to_lowercase()
            .contains(settings.owner_name.to_lowercase().as_str())
            || target_user.is_admin
        {
            return Err(TinyBoardsError::from_message(
                403,
                "can't purge a site admin or the owner",
            ));
        }

        // purge user banner
        if let Some(banner) = target_person.banner {
            purge_local_image_by_url(context.pool(), &banner).await.ok();
        }

        // purge user avatar
        if let Some(avatar) = target_person.avatar {
            purge_local_image_by_url(context.pool(), &avatar).await.ok();
        }

        // purge all submitted media/images from user
        purge_local_image_posts_for_user(
            target_person_id,
            context.pool(),
        )
        .await?;

        // delete person & local user
        Person::delete(context.pool(), target_person_id).await?;
        LocalUser::delete(context.pool(), target_user.id.clone()).await?;

        let reason = data.reason.clone();

        let form = AdminPurgeUserForm {
            admin_id: view.person.id,
            person_id: target_person_id.clone(),
            reason: Some(reason),
        };

        let mod_action = AdminPurgeUser::create(context.pool(), &form).await?;

        Ok(ModActionResponse { mod_action })
    }
}
