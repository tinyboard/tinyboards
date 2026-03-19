use crate::{DbPool, Settings, storage::StorageBackend};
use crate::storage::image_processing::{process_upload as process_image, ImageProcessingSettings};
use async_graphql::*;
use diesel_async::RunQueryDsl;
use std::io::Read;
use tinyboards_db::{
    models::upload::UploadInsertForm,
    schema::uploads,
    utils::get_conn,
};
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
use uuid::Uuid;

/// MIME types that the image processor can handle.
const IMAGE_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

pub async fn upload_file(
    upload: Upload,
    file_name: Option<String>,
    for_user_id: Uuid,
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

    let upload_form = UploadInsertForm {
        user_id: for_user_id,
        original_name: original_file_name.clone(),
        file_name: generated_file_name,
        file_path: path,
        upload_url: upload_url.to_string(),
        size_bytes: size,
        thumbnail_url: None,
        optimized_url: None,
        processing_status: Some("pending".to_string()),
    };

    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(uploads::table)
        .values(&upload_form)
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(format!("Failed to save upload record: {}", e)))?;

    tracing::info!("File uploaded successfully: {} ({})", upload_url, format_file_size(size));
    Ok(upload_url)
}

pub async fn upload_file_opendal(
    upload: Upload,
    file_name: Option<String>,
    for_user_id: Uuid,
    max_size_mb: Option<u32>,
    ctx: &Context<'_>,
) -> Result<Url> {
    let _settings = ctx.data::<Settings>()?.as_ref();
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

    // Read all bytes for validation and (for images) processing
    let file = upload_value.content;
    let mut async_reader = tokio::fs::File::from_std(file);
    let mut all_bytes = Vec::new();
    async_reader.read_to_end(&mut all_bytes).await?;

    let total_size = all_bytes.len() as i64;

    // Check size limit
    if let Some(max_mb) = max_size_mb {
        let max_size = (max_mb * 1024 * 1024) as i64;
        if total_size > max_size {
            return Err(TinyBoardsError::from_message(
                400,
                &format!("File exceeds maximum size of {}", format_file_size(max_size)),
            ).into());
        }
    }

    // Validate file content matches declared type
    validate_file_content(&all_bytes, &content_type).map_err(|e| {
        TinyBoardsError::from_message(400, &e)
    })?;

    // Determine the actual content type and bytes to store.
    // For images, run through the processing pipeline (resize, strip EXIF, convert).
    let is_image = IMAGE_MIME_TYPES.contains(&content_type.as_str());

    let (store_bytes, store_mime, thumbnail_bytes) = if is_image {
        let img_settings = ImageProcessingSettings::default();
        let processed = process_image(&all_bytes, &content_type, &img_settings)
            .map_err(|e| -> async_graphql::Error { e.into() })?;

        tracing::info!(
            "Image processed: {}x{}, {} -> {} bytes",
            processed.width, processed.height,
            processed.original_size, processed.processed_size
        );

        (processed.data, processed.mime_type, processed.thumbnail_data)
    } else {
        (all_bytes, content_type.clone(), None)
    };

    // Generate secure filename using the (possibly updated) mime type
    let generated_file_name = generate_secure_filename(
        file_name.or(Some(original_file_name.clone())),
        &store_mime
    );

    // Determine storage key (subdirectory + filename)
    let storage_key = get_storage_key_for_type(&generated_file_name, &store_mime, &original_file_name);

    // Store the processed file
    let final_size = store_bytes.len() as i64;
    storage.write(&storage_key, store_bytes).await
        .map_err(|e| -> async_graphql::Error { e.into() })?;

    // Store thumbnail if one was generated
    let mut thumb_url = None;
    if let Some(thumb_bytes) = thumbnail_bytes {
        let thumb_key = make_thumbnail_key(&storage_key);
        match storage.write(&thumb_key, thumb_bytes).await {
            Ok(()) => {
                thumb_url = Some(storage.get_public_url(&thumb_key));
            }
            Err(e) => {
                // Log but don't fail the upload if the thumbnail write fails
                tracing::error!("Failed to write thumbnail {}: {}", thumb_key, e);
            }
        }
    }

    // Generate public URL
    let upload_url = Url::parse(&storage.get_public_url(&storage_key))?;

    // Image processing is currently synchronous — it completes before the response
    // is sent. Both image and non-image uploads are marked "complete" immediately.
    // The processing_status column exists to support future async processing: if
    // upload latency becomes an issue, processing can be moved to a background task
    // with the initial status set to "pending" and updated on completion/failure.
    let (optimized_url, processing_status) = if is_image {
        (Some(upload_url.to_string()), Some("complete".to_string()))
    } else {
        (None, Some("complete".to_string()))
    };

    // Save to database
    let upload_form = UploadInsertForm {
        user_id: for_user_id,
        original_name: original_file_name.clone(),
        file_name: generated_file_name,
        file_path: storage_key.clone(),
        upload_url: upload_url.to_string(),
        size_bytes: final_size,
        thumbnail_url: thumb_url,
        optimized_url,
        processing_status,
    };

    let conn = &mut get_conn(pool).await?;
    diesel::insert_into(uploads::table)
        .values(&upload_form)
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(format!("Failed to save upload record: {}", e)))?;

    tracing::info!("File uploaded: {} ({} bytes)", storage_key, final_size);

    Ok(upload_url)
}

/// Generate a thumbnail storage key from the original key.
/// Example: "images/abc123.webp" -> "images/abc123_thumb.webp"
fn make_thumbnail_key(key: &str) -> String {
    match key.rsplit_once('.') {
        Some((base, ext)) => format!("{}_thumb.{}", base, ext),
        None => format!("{}_thumb", key),
    }
}

/// Placeholder — file deletion is not yet implemented in the rewritten codebase.
#[allow(dead_code)]
pub async fn delete_file(_pool: &DbPool, _upload_url: &str) -> Result<(), TinyBoardsError> {
    // TODO: implement using uploads::table query once the old trait-based methods are removed
    Ok(())
}
