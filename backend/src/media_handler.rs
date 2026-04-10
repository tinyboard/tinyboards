use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web::http::header;
use tinyboards_api::context::TinyBoardsContext;
use tinyboards_utils::error::TinyBoardsError;
use std::path::PathBuf;
use tokio::fs;

/// Serves media files from the configured storage backend.
/// Tries the active storage backend first, then falls back to the local filesystem
/// so that files uploaded before a backend migration are still accessible.
/// Supports HTTP Range requests for video seeking.
pub async fn serve_media(
    context: web::Data<TinyBoardsContext>,
    req: HttpRequest,
) -> Result<HttpResponse, TinyBoardsError> {
    // Extract the file path from the request
    let path = req.match_info().query("filename");

    // Remove leading slash if present
    let storage_key = path.trim_start_matches('/');

    tracing::debug!("Serving media file: {}", storage_key);

    // Try the configured storage backend first
    let data = match context.storage().read(storage_key).await {
        Ok(data) => data,
        Err(_) => {
            // Storage backend didn't have it — check local filesystem as fallback.
            // This handles the migration case where old files live on disk but the
            // active backend has been switched to S3/Wasabi/etc.
            let media_path = context.settings().get_media_path();
            let local_path = PathBuf::from(&media_path).join(storage_key);

            tracing::debug!("File not in storage backend, trying local path: {:?}", local_path);
            fs::read(&local_path).await.map_err(|_| {
                tracing::warn!("File not found in storage backend or local filesystem: {}", storage_key);
                TinyBoardsError::from_message(404, "File not found")
            })?
        }
    };

    let content_type = get_content_type(storage_key);
    let total_len = data.len();

    // Parse Range header for partial content (needed for video seeking)
    if let Some(range_header) = req.headers().get(header::RANGE) {
        if let Ok(range_str) = range_header.to_str() {
            if let Some(range) = parse_range(range_str, total_len) {
                let (start, end) = range;
                let slice = data[start..=end].to_vec();
                return Ok(HttpResponse::PartialContent()
                    .content_type(content_type)
                    .insert_header((header::CONTENT_LENGTH, (end - start + 1).to_string()))
                    .insert_header((header::ACCEPT_RANGES, "bytes"))
                    .insert_header((
                        header::CONTENT_RANGE,
                        format!("bytes {}-{}/{}", start, end, total_len),
                    ))
                    .body(slice));
            }
        }
    }

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .insert_header((header::CONTENT_LENGTH, total_len.to_string()))
        .insert_header((header::ACCEPT_RANGES, "bytes"))
        .body(data))
}

/// Parse a simple "bytes=start-end" range header.
/// Returns (start, end) inclusive, or None if the range is invalid.
fn parse_range(range_str: &str, total: usize) -> Option<(usize, usize)> {
    let range_str = range_str.strip_prefix("bytes=")?;
    let mut parts = range_str.splitn(2, '-');
    let start_str = parts.next()?.trim();
    let end_str = parts.next()?.trim();

    if start_str.is_empty() {
        // Suffix range: bytes=-500 means last 500 bytes
        let suffix_len: usize = end_str.parse().ok()?;
        let start = total.saturating_sub(suffix_len);
        Some((start, total - 1))
    } else {
        let start: usize = start_str.parse().ok()?;
        let end = if end_str.is_empty() {
            total - 1
        } else {
            end_str.parse::<usize>().ok()?.min(total - 1)
        };
        if start <= end && start < total {
            Some((start, end))
        } else {
            None
        }
    }
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
