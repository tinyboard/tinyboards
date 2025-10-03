use actix_web::{web, HttpRequest, HttpResponse, Result};
use tinyboards_api::context::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use std::path::PathBuf;
use tokio::fs;

/// Serves media files from the configured storage backend
/// First checks local filesystem for backwards compatibility, then falls back to storage backend
pub async fn serve_media(
    context: web::Data<TinyBoardsContext>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError> {
    // Extract the file path from the request
    let path = req.match_info().query("filename");

    // Remove leading slash if present
    let storage_key = path.trim_start_matches('/');

    tracing::debug!("Serving media file: {}", storage_key);

    // First, check if file exists locally (for backwards compatibility with existing uploads)
    let media_path = context.settings().get_media_path();
    let local_path = PathBuf::from(&media_path).join(storage_key);

    if local_path.exists() {
        tracing::debug!("Found file locally at: {:?}", local_path);
        match fs::read(&local_path).await {
            Ok(data) => {
                let content_type = get_content_type(storage_key);
                return Ok(HttpResponse::Ok()
                    .content_type(content_type)
                    .body(data));
            }
            Err(e) => {
                tracing::warn!("File exists but failed to read locally: {:?}", e);
                // Fall through to storage backend
            }
        }
    }

    // If not found locally, try the configured storage backend
    tracing::debug!("File not found locally, checking storage backend");
    let data = context.storage().read(storage_key).await
        .map_err(|e| {
            tracing::error!("Failed to read file from storage backend: {:?}", e);
            TinyBoardsError::from_message(404, "File not found")
        })?;

    // Determine content type from file extension
    let content_type = get_content_type(storage_key);

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .body(data))
}

/// Determine content type from file extension
fn get_content_type(path: &str) -> &'static str {
    let extension = path.rsplit('.').next().unwrap_or("");

    match extension.to_lowercase().as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",

        // Videos
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        "mov" => "video/quicktime",

        // Audio
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",

        // Documents
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "json" => "application/json",

        // Default
        _ => "application/octet-stream",
    }
}
