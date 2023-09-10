use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    admin::LeaveAdmin,
    utils::require_user, 
    site::GetSiteResponse
};
use tinyboards_db::{
    models::{
        apub::actor_language::SiteLanguage,
        apub::language::Language,
        moderator::mod_actions::{ModAddAdmin, ModAddAdminForm},
        person::person::{Person, PersonForm},
    },
    traits::Crud, utils::naive_now
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

        let view = require_user(
            context.pool(), 
            context.master_key(), 
            auth
        )
        .await
        .require_admin()
        .unwrap()?;
        
        let admins = PersonView::admins(context.pool()).await?;
        if admins.len() == 1 {
            return Err(TinyBoardsError::from_message(400, "cannot leave admin if you are the only admin."));
        }

        let person_id = view.person.id;
        
        let form = PersonForm {
            is_admin: Some(false),
            updated: Some(naive_now()),
            ..PersonForm::default()
        };
        Person::update(context.pool(), person_id, &form).await?;

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