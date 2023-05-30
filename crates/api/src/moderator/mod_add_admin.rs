use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{AddAdmin, ModActionResponse},
    utils::require_user,
};
use tinyboards_db::{
    models::{
        moderator::mod_actions::{ModAddAdmin, ModAddAdminForm},
        user::users::User,
    },
    traits::Crud,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for AddAdmin {
    type Response = ModActionResponse<ModAddAdmin>;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &AddAdmin = &self;

        // require admin to add new admin
        // TODO - reconfigure this logic to only allow site owner to add new admin
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin()
            .unwrap()?;

        let added = data.added;
        let added_person_id = data.added_person_id;

        // update added user to be an admin
        User::update_admin(context.pool(), added_person_id.clone(), added.clone()).await?;

        // log this mod action
        let mod_add_admin_form = ModAddAdminForm {
            mod_person_id: user.id,
            other_person_id: added_person_id.clone(),
            removed: Some(Some(!added.clone())),
        };

        // submit to the mod log
        let mod_action = ModAddAdmin::create(context.pool(), &mod_add_admin_form).await?;

        Ok(ModActionResponse { mod_action })
    }
}
