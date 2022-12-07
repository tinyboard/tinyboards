use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    moderator::{AddAdmin, ModActionResponse},
    utils::{blocking, require_user},
};
use tinyboards_db::{
    models::{moderator::mod_actions::{ModAddAdmin, ModAddAdminForm}, user::user::User},
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
        let added_user_id = data.added_user_id;
        
        // update added user to be an admin
        blocking(context.pool(), move |conn| {
            User::update_admin(conn, added_user_id.clone(), added.clone())
        })
        .await??;

        // log this mod action
        let mod_add_admin_form = ModAddAdminForm {
            mod_user_id: user.id,
            other_user_id: added_user_id.clone(),
            removed: Some(Some(!added.clone()))
        };

        // submit to the mod log
        let mod_action = blocking(context.pool(), move |conn| {
            ModAddAdmin::create(conn, &mod_add_admin_form)
        })
        .await??;

        Ok(ModActionResponse { mod_action })
    }
}