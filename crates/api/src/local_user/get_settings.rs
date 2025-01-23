use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{GetUserSettings, GetUserSettingsResponse},
    utils::{
        require_user,
    },
};
use tinyboards_db_views::structs::LocalUserSettingsView;
use tinyboards_utils::{error::TinyBoardsError};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetUserSettings {
    type Response = GetUserSettingsResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<Self::Response, TinyBoardsError> {

        let local_user_view = 
            require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
            
        let settings = LocalUserSettingsView::read(context.pool(), local_user_view.person.id).await?;
        
        Ok( GetUserSettingsResponse { settings } )
    }
}