use crate::{DbPool, Settings};
use async_graphql::*;
use std::io::Read;
use tinyboards_db::models::site::uploads::{Upload as DbUpload, UploadForm};
use tinyboards_db::newtypes::DbUrl;
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
    max_size_mb: Option<u32>,
    ctx: &Context<'_>,
) -> Result<Url> {
    let settings = ctx.data::<Settings>()?.as_ref();
    let pool = ctx.data::<DbPool>()?;
    let media_path = settings.get_media_path();

    let mut file_bytes: Vec<u8> = Vec::new();

    let upload_value = upload.value(ctx)?;
    let original_file_name = upload_value.filename;
    let content_type = upload_value.content_type.unwrap_or(String::new());
    // TODO: read max allowed file size from either the config or the site settings instead
    let max_size = (max_size_mb.unwrap_or(50) * 1024 * 1024) as i64;

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
    let path = format!("{}/{}", media_path, file_name);

    println!("got here, saving to {:?}", &path);
    upload
        .value(ctx)?
        .into_read()
        .read_to_end(&mut file_bytes)?;
    //let _ = File::create("hi.txt").await?;
    let mut file = File::create(&path).await?;
    file.write_all(&file_bytes).await?;
    file.flush().await?;

    let size = file_bytes.len().try_into().unwrap();
    println!("File size is {}", size);
    if size > max_size {
        // File too large! Delete it
        // Files exceeding the absolute maximum will be rejected by proxy before even hitting the server
        if let Err(_) = std::fs::remove_file(path.clone()) {
            eprintln!("File {} exceeds maximum allowed size of {}MB, but couldn't be deleted automatically. Please delete it.", path, max_size);
        }

        return Err(TinyBoardsError::from_message(
            400,
            &format!("File exceeds maximum allowed size of {}MB.", max_size),
        )
        .into());
    }

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
        size,
    };

    let _upload = DbUpload::create(pool, &upload_form).await?;

    //let upload_url = Url::parse(upload_url)?;
    Ok(upload_url)
}

pub async fn delete_file(pool: &DbPool, img_url: &DbUrl) -> Result<(), TinyBoardsError> {
    let file = DbUpload::find_by_url(pool, img_url).await?;

    // delete the file from the file system
    std::fs::remove_file(file.file_path.clone())?;

    // delete DB entry
    DbUpload::delete(pool, file.id.clone()).await?;

    Ok(())
}
