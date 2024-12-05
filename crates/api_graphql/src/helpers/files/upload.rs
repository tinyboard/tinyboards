use crate::{DbPool, Settings};
use async_graphql::*;
use std::io::Read;
use tinyboards_db::models::site::uploads::{Upload as DbUpload, UploadForm};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{
    error::TinyBoardsError,
    utils::{generate_rand_string, get_file_type, is_acceptable_file_type},
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

pub async fn upload_file(
    upload: Upload,
    file_name: Option<String>,
    for_person_id: i32,
    ctx: &Context<'_>,
) -> Result<Url> {
    let settings = ctx.data::<Settings>()?.as_ref();
    let pool = ctx.data::<DbPool>()?;
    let media_path = settings.get_media_path();

    let mut file_bytes: Vec<u8> = Vec::new();

    let upload_value = upload.value(ctx)?;
    let original_file_name = upload_value.filename;
    let content_type = upload_value.content_type.unwrap_or(String::new());

    if !is_acceptable_file_type(&content_type) {
        return Err(TinyBoardsError::from_message(
            500,
            &format!("{} is not an acceptable file type", content_type),
        )
        .into());
    }

    let file_type = get_file_type(&content_type);
    let file_name = format!(
        "{}.{}",
        match file_name {
            Some(file_name) => file_name,
            None => generate_rand_string(),
        },
        file_type
    );
    let path = format!("{}/{}", media_path, &file_name);

    upload
        .value(ctx)?
        .into_read()
        .read_to_end(&mut file_bytes)?;
    let mut file = File::create(&path).await?;
    file.write(&file_bytes).await?;

    let upload_url = Url::parse(&format!(
        "{}/media/{}",
        settings.get_protocol_and_hostname(),
        &file_name
    ))?;

    let upload_form = UploadForm {
        person_id: for_person_id,
        original_name: original_file_name,
        file_name: file_name,
        file_path: path,
        upload_url: Some(upload_url.clone().into()),
        size: file_bytes.len().try_into().unwrap(),
    };

    let _upload = DbUpload::create(pool, &upload_form).await?;

    //let upload_url = Url::parse(upload_url)?;
    Ok(upload_url)
}
