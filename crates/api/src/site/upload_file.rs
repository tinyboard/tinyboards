use crate::PerformUpload;
use actix_multipart::Multipart;
use actix_web::web::Data;
use tinyboards_api_common::utils::require_user;
use tinyboards_api_common::{
    data::TinyBoardsContext,
};
use tinyboards_db::models::{site::password_resets::*, user::users::User};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{error::TinyBoardsError, utils::generate_rand_string};
use futures::{StreamExt, TryStreamExt};

#[async_trait::async_trait(?Send)]
impl<'des> PerformUpload<'des> for Multipart {

    type Response = ();
    type Route = ();

    async fn perform_upload(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<(), TinyBoardsError> {


        // require there be a logged in user to perform the upload
        let user = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;


        let mut data = self;

        while let Some(item) = data.next().await {
            let mut field = item.unwrap();
            let content_disposition = field.content_disposition();
            let file_name = content_disposition.get_filename().unwrap();
            let content_type = field.content_type().unwrap().to_string();

            let mut file_bytes: Vec<u8> = Vec::new();

            while let Some(chunk) = field.next().await {
                let chunk_data = chunk.unwrap();
                file_bytes.extend_from_slice(&chunk_data);
            }

        }
        
        // next: save file_bytes to a locally stored file with a randomized name
        // next: save file metadata to the database
        // next: send response with the uploaded files url


        todo!()
    }
}
