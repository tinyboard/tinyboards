use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
  data::TinyBoardsContext,
  person::{DeleteAccount, DeleteAccountResponse},
  utils::require_user,
};
use tinyboards_utils::{error::TinyBoardsError, passhash::verify_password};

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteAccount {
    type Response = DeleteAccountResponse;
    type Route = ();

    #[tracing::instrument(skip_all)]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: &DeleteAccount = &self;
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let valid = verify_password(&view.local_user.passhash, &data.password);

        if !valid {
            return Err(TinyBoardsError::from_message(400, "invalid password"));
        }

        Ok(DeleteAccountResponse {})
    }
}