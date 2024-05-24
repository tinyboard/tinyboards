use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    admin::LeaveAdmin, data::TinyBoardsContext, site::GetSiteResponse, utils::require_user,
};
use tinyboards_db::{
    models::{
        apub::{actor_language::SiteLanguage, language::Language},
        moderator::mod_actions::{ModAddAdmin, ModAddAdminForm},
        person::{local_user::LocalUser, person::Person},
    },
    traits::Crud,
};
use tinyboards_db_views::structs::{PersonView, SiteView};
use tinyboards_utils::{error::TinyBoardsError, version};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for LeaveAdmin {
    type Response = GetSiteResponse;
    type Route = ();

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            //.require_admin()
            .unwrap()?;

        if view.local_user.admin_level < 1 {
            return Err(TinyBoardsError::from_message(
                403,
                "You cannot step leave admin if you're not even one.",
            ));
        }

        if view.local_user.is_owner() {
            return Err(TinyBoardsError::from_message(400, "You cannot leave admin because you're the owner of this instance. You must transfer ownership to someone else before you can step down."));
        }

        let admins = PersonView::admins(context.pool()).await?;
        if admins.len() == 1 {
            return Err(TinyBoardsError::from_message(
                400,
                "cannot leave admin if you are the only admin.",
            ));
        }

        LocalUser::update_admin(context.pool(), view.local_user.id, 0).await?;

        let person_id = view.person.id;
        Person::update_admin(context.pool(), person_id, 0).await?;

        let form = ModAddAdminForm {
            mod_person_id: person_id,
            other_person_id: person_id,
            removed: Some(Some(true)),
        };
        ModAddAdmin::create(context.pool(), &form).await?;

        let site_view = SiteView::read_local(context.pool()).await?;
        let admins = PersonView::admins(context.pool()).await?;
        let all_languages = Language::read_all(context.pool()).await?;
        let discussion_languages = SiteLanguage::read_local_raw(context.pool()).await?;

        Ok(GetSiteResponse {
            site_view,
            admins,
            version: version::VERSION.to_string(),
            all_languages,
            discussion_languages,
        })
    }
}
