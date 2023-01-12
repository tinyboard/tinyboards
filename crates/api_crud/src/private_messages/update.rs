use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    private_messages::{CreatePrivateMessage, CreatePrivateMessageResponse, EditPrivateMessage, PrivateMessageResponse},
    utils::{
        blocking,
        require_user, check_user_block,
    },
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::{
        user::private_messages::{PrivateMessage, PrivateMessageForm},
    },
    traits::Crud,
};
use tinyboards_db_views::structs::PrivateMessageView;
use tinyboards_utils::{parser::parse_markdown, TinyBoardsError};


#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditPrivateMessage {
    type Response = PrivateMessageResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<PrivateMessageResponse, TinyBoardsError> {

        todo!()
    }
}