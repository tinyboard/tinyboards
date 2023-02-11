use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{GetUserSettings, GetUserSettingsResponse},
    utils::{
        get_user_view_from_jwt,
    },
};
use tinyboards_db_views::structs::UserSettingsView;
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

        let user_view = 
            get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let settings = UserSettingsView::read(context.pool(), user_view.user.id).await?;
        
        Ok( GetUserSettingsResponse { settings } )
    }
}