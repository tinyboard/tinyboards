use crate::{DbPool, Settings, storage::StorageBackend};
use async_graphql::*;
use std::io::Read;
use tinyboards_db::models::site::uploads::{Upload as DbUpload, UploadForm};
use tinyboards_db::newtypes::DbUrl;
use tinyboards_db::traits::Crud;
use tinyboards_utils::{
    error::TinyBoardsError,
    utils::{
        generate_secure_filename, is_acceptable_file_type,
        validate_file_size, validate_file_content, get_file_path_for_type,
        get_storage_key_for_type, ensure_upload_directories, format_file_size
    },
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
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

pub async fn upload_file_opendal(
    upload: Upload,
    file_name: Option<String>,
    for_user_id: i32,
    max_size_mb: Option<u32>,
    ctx: &Context<'_>,
) -> Result<Url> {
    let settings = ctx.data::<Settings>()?.as_ref();
    let pool = ctx.data::<DbPool>()?;
    let storage = ctx.data::<StorageBackend>()?;

    let upload_value = upload.value(ctx)?;
    let original_file_name = upload_value.filename.clone();
    let content_type = upload_value.content_type.clone().unwrap_or_default();

    // Validate file type
    if !is_acceptable_file_type(&content_type) {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("{} is not an acceptable file type", content_type),
        ).into());
    }

    // For validation, read first chunk
    let file = upload_value.content;
    let mut async_reader = tokio::fs::File::from_std(file);

    let mut validation_buffer = vec![0u8; 1024 * 1024]; // 1MB for validation
    let bytes_read = async_reader.read(&mut validation_buffer).await?;
    validation_buffer.truncate(bytes_read);

    // Validate file content
    validate_file_content(&validation_buffer, &content_type).map_err(|e| {
        TinyBoardsError::from_message(400, &e)
    })?;

    // Generate secure filename
    let generated_file_name = generate_secure_filename(
        file_name.or(Some(original_file_name.clone())),
        &content_type
    );

    // Determine storage key (subdirectory + filename)
    let storage_key = get_storage_key_for_type(&generated_file_name, &content_type, &original_file_name);

    // Stream upload with OpenDAL
    let mut writer = storage.operator().writer_with(&storage_key)
        .chunk(8 * 1024 * 1024)  // 8MB chunks
        .concurrent(4)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create writer"))?;

    // Write validation buffer first
    let mut total_size = validation_buffer.len() as i64;
    writer.write(validation_buffer).await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to write"))?;

    // Stream remaining content
    let mut buffer = vec![0u8; 8 * 1024 * 1024];
    loop {
        let n = async_reader.read(&mut buffer).await?;
        if n == 0 { break; }

        total_size += n as i64;

        // Check size limit during streaming
        if let Some(max_mb) = max_size_mb {
            let max_size = (max_mb * 1024 * 1024) as i64;
            if total_size > max_size {
                writer.abort().await.ok();
                return Err(TinyBoardsError::from_message(
                    400,
                    &format!("File exceeds maximum size of {}", format_file_size(max_size)),
                ).into());
            }
        }

        // Clone the buffer slice to create owned data
        let chunk = buffer[..n].to_vec();
        writer.write(chunk).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to write"))?;
    }

    writer.close().await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to complete upload"))?;

    // Generate public URL
    let upload_url = Url::parse(&storage.get_public_url(&storage_key))?;

    // Save to database
    let upload_form = UploadForm {
        user_id: for_user_id,
        original_name: original_file_name.clone(),
        file_name: generated_file_name,
        file_path: storage_key.clone(),  // Stores logical key, not filesystem path
        upload_url: Some(upload_url.clone().into()),
        size: total_size,
    };

    DbUpload::create(pool, &upload_form).await?;

    tracing::info!("File uploaded: {} ({} bytes)", storage_key, total_size);

    Ok(upload_url)
}

pub async fn delete_file(pool: &DbPool, img_url: &DbUrl) -> Result<(), TinyBoardsError> {
    let file = DbUpload::find_by_url(pool, img_url).await?;

    // delete the file from the file system
    // Ignore error if file doesn't exist (it may have been manually deleted)
    if let Err(e) = std::fs::remove_file(file.file_path.clone()) {
        if e.kind() != std::io::ErrorKind::NotFound {
            // Only propagate errors that aren't "file not found"
            return Err(e.into());
        }
        // File doesn't exist, that's fine - continue to delete DB entry
    }

    // delete DB entry
    DbUpload::delete(pool, file.id.clone()).await?;

    Ok(())
}
