use crate::PerformUpload;
use actix_multipart::Multipart;
use actix_web::web::Data;
use tinyboards_api_common::utils::require_user;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::FileUploadResponse,
};
use tinyboards_db::models::{site::uploads::*};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{error::TinyBoardsError, utils::{generate_rand_string, is_acceptable_file_type, get_file_type}, };
use futures::StreamExt;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
};
use url::Url;

#[async_trait::async_trait(?Send)]
impl<'des> PerformUpload<'des> for Multipart {

    type Response = FileUploadResponse;
    type Route = ();

    async fn perform_upload(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<FileUploadResponse, TinyBoardsError> {

        // require there be a logged in user to perform the upload
        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let mut data = self;
        let mut uploads = Vec::new();

        while let Some(Ok(mut field)) = data.next().await {
            //println!("{:#?}", item);

            //let mut field = item.unwrap();
            let content_disposition = field.content_disposition().clone();
            let original_file_name = content_disposition.get_filename().clone().unwrap();
            let content_type = field.content_type().unwrap().to_string();

            if !is_acceptable_file_type(&content_type) {
                return Err(TinyBoardsError::from_message(500, "invalid file type"));
            } else {

                let mut file_bytes: Vec<u8> = Vec::new();
    
                while let Some(chunk) = field.next().await {
                    let chunk_data = chunk.unwrap();
                    file_bytes.extend_from_slice(&chunk_data);
                }

                let file_type = get_file_type(&content_type);
                let file_name = format!("{}.{}", generate_rand_string(), file_type);

                let media_path = context.settings().get_media_path();

                let file_path = format!("{}/{}", &media_path, &file_name);
                let upload_url = format!("{}/media/{}", context.settings().get_protocol_and_hostname(), file_name.clone());
                let mut file = File::create(&file_path).await?;
                file.write_all(&file_bytes).await?;

                let upload_form = UploadForm {
                    person_id: view.person.id,
                    original_name: original_file_name.to_string(),
                    file_name: file_name.clone(),
                    file_path: file_path.clone(),
                    upload_url: Some(Url::parse(&upload_url)?.into()),
                    size: file_bytes.len().try_into().unwrap()
                };

                let upload = Upload::create(context.pool(), &upload_form).await?;

                uploads.push(upload.upload_url.clone());
            }
        }

        Ok(FileUploadResponse { uploads })
    }
}
