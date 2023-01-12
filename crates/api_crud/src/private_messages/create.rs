use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    private_messages::{CreatePrivateMessage, CreatePrivateMessageResponse},
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
impl <'des> PerformCrud<'des> for CreatePrivateMessage {
    type Response = CreatePrivateMessageResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<CreatePrivateMessageResponse, TinyBoardsError> {
        
        let data: &CreatePrivateMessage = &self;

        let sender 
            = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        // error out if the recipient is blocking you
        check_user_block(sender.id.clone(), data.recipient_id.clone(), context.pool())
            .await?;

        let creator_id = sender.id.clone();
        let recipient_id = data.recipient_id.clone();
        let subject = data.subject.clone();
        let body = data.body.clone();
        let body_parsed = parse_markdown(&body.as_str());

        let private_message_form = PrivateMessageForm {
            creator_id: Some(creator_id), 
            recipient_id: Some(recipient_id),
            subject: Some(subject),
            body: body_parsed,
            is_deleted: Some(false),
            read: Some(false),
            updated: None,
        };

        // create the private message
        let pm = blocking(context.pool(), move |conn| {
            PrivateMessage::create(conn, &private_message_form)
        })
        .await??;

        let message = blocking(context.pool(), move |conn| {
            PrivateMessageView::read(conn, pm.id)
        })
        .await??;


        // eventually add email support here and ws stuff

        Ok(CreatePrivateMessageResponse { message })
    }
}