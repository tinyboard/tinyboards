use crate::{DbPool, Settings};
use async_graphql::*;
use std::io::Read;
use tinyboards_db::models::site::uploads::{Upload as DbUpload, UploadForm};
use tinyboards_db::newtypes::DbUrl;
use tinyboards_db::traits::Crud;
use tinyboards_utils::{
    error::TinyBoardsError,
    utils::{
        generate_secure_filename, get_file_type_extended, is_acceptable_file_type,
        validate_file_size, validate_file_content, get_file_path_for_type,
        get_file_url, ensure_upload_directories, format_file_size
    },
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

pub async fn upload_file(
    upload: Upload,
    file_name: Option<String>,
    for_user_id: i32,
    max_size_mb: Option<u32>,
    ctx: &Context<'_>,
) -> Result<Url> {
    let settings = ctx.data::<Settings>()?.as_ref();
    let pool = ctx.data::<DbPool>()?;
    let media_path = settings.get_media_path();

    // Ensure upload directories exist
    ensure_upload_directories(&media_path).await.map_err(|e| {
        TinyBoardsError::from_message(500, &format!("Failed to create upload directories: {}", e))
    })?;

    let mut file_bytes: Vec<u8> = Vec::new();

    let upload_value = upload.value(ctx)?;
    let original_file_name = upload_value.filename;
    let content_type = upload_value.content_type.unwrap_or(String::new());

    // Enhanced file type validation
    if !is_acceptable_file_type(&content_type) {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("{} is not an acceptable file type", content_type),
        )
        .into());
    }

    // Read file bytes first for validation
    upload
        .value(ctx)?
        .into_read()
        .read_to_end(&mut file_bytes)?;

    let size = file_bytes.len() as i64;

    // Enhanced size validation with type-specific limits
    if let Some(max_mb) = max_size_mb {
        let max_size = (max_mb * 1024 * 1024) as i64;
        if size > max_size {
            return Err(TinyBoardsError::from_message(
                400,
                &format!("File exceeds maximum allowed size of {}", format_file_size(max_size)),
            )
            .into());
        }
    } else {
        // Use type-specific validation
        validate_file_size(&content_type, size).map_err(|e| {
            TinyBoardsError::from_message(400, &e)
        })?;
    }

    // Validate file content matches declared type
    validate_file_content(&file_bytes, &content_type).map_err(|e| {
        TinyBoardsError::from_message(400, &e)
    })?;

    // Generate secure filename
    let generated_file_name = generate_secure_filename(
        file_name.or(Some(original_file_name.clone())),
        &content_type
    );

    // Get appropriate file path based on type and content
    let path = get_file_path_for_type(&media_path, &generated_file_name, &content_type);

    // Ensure the specific subdirectory exists
    if let Some(parent) = std::path::Path::new(&path).parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            TinyBoardsError::from_message(500, &format!("Failed to create directory: {}", e))
        })?;
    }

    println!("Saving file to: {}", &path);
    let mut file = File::create(&path).await?;
    file.write_all(&file_bytes).await?;
    file.flush().await?;

    // Generate URL with proper subdirectory structure
    let url_path = if path.contains("/emojis/") {
        format!("emojis/{}", generated_file_name)
    } else if path.contains("/avatars/") {
        format!("avatars/{}", generated_file_name)
    } else if path.contains("/videos/") {
        format!("videos/{}", generated_file_name)
    } else if path.contains("/audio/") {
        format!("audio/{}", generated_file_name)
    } else if path.contains("/documents/") {
        format!("documents/{}", generated_file_name)
    } else {
        generated_file_name.clone()
    };

    let upload_url = Url::parse(&format!(
        "{}/media/{}",
        settings.get_protocol_and_hostname(),
        url_path
    ))?;

    let upload_form = UploadForm {
        user_id: for_user_id,
        original_name: original_file_name.clone(),
        file_name: generated_file_name,
        file_path: path,
        upload_url: Some(upload_url.clone().into()),
        size,
    };

    let _upload = DbUpload::create(pool, &upload_form).await?;

    println!("File uploaded successfully: {} ({})", upload_url, format_file_size(size));
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
