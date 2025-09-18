use crate::{DbPool, Settings};
use tinyboards_db::models::site::site::Site;
use async_graphql::*;
use std::io::Read;
use tinyboards_db::models::{
    emoji::emoji::Emoji,
    site::uploads::{Upload as DbUpload, UploadForm},
};
use tinyboards_db::traits::Crud;
use tinyboards_utils::{
    error::TinyBoardsError,
    utils::{
        generate_secure_filename, get_file_type_extended,
        validate_file_content, format_file_size
    },
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

const MAX_EMOJI_DIMENSIONS: u32 = 512; // 512x512 max dimensions (for future use)

pub async fn upload_emoji_file(
    upload: Upload,
    shortcode: String,
    for_user_id: i32,
    ctx: &Context<'_>,
) -> Result<Url> {
    let settings = ctx.data::<Settings>()?.as_ref();
    let pool = ctx.data::<DbPool>()?;
    let media_path = settings.get_media_path();

    // Load site configuration for emoji file size limit
    let site_config = Site::read(pool).await?;
    let max_size_mb = site_config.emoji_max_file_size_mb as u32;

    let mut file_bytes: Vec<u8> = Vec::new();

    let upload_value = upload.value(ctx)?;
    let content_type = upload_value.content_type.unwrap_or(String::new());
    let max_size = (max_size_mb * 1024 * 1024) as i64;

    // Validate file type - only allow image types for emojis
    if !is_emoji_content_type(&content_type) {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("{} is not a valid emoji file type. Only PNG, JPG, GIF, and WebP are allowed", content_type),
        )
        .into());
    }

    // Validate file content matches declared type
    validate_file_content(&file_bytes, &content_type).map_err(|e| {
        TinyBoardsError::from_message(400, &e)
    })?;

    // Generate secure emoji filename with shortcode
    let file_name = format!("emoji_{}", generate_secure_filename(Some(shortcode.clone()), &content_type));
    let path = format!("{}/emojis/{}", media_path, file_name);

    // Ensure the emojis directory exists
    let emoji_dir = format!("{}/emojis", media_path);
    tokio::fs::create_dir_all(&emoji_dir).await?;

    // Read file bytes
    upload
        .value(ctx)?
        .into_read()
        .read_to_end(&mut file_bytes)?;

    // Check file size
    let size = file_bytes.len() as i64;
    if size > max_size {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("Emoji file exceeds maximum allowed size of {}", format_file_size(max_size)),
        )
        .into());
    }

    // Validate image dimensions
    if let Err(e) = validate_emoji_dimensions(&file_bytes, &content_type) {
        return Err(e.into());
    }

    // Write file to disk
    let mut file = File::create(&path).await?;
    file.write_all(&file_bytes).await?;
    file.flush().await?;

    let upload_url = Url::parse(&format!(
        "{}/media/emojis/{}",
        settings.get_protocol_and_hostname(),
        &file_name
    ))?;

    // Record upload in database
    let upload_form = UploadForm {
        user_id: for_user_id,
        original_name: format!("{}.{}", shortcode, get_file_type_extended(&content_type)),
        file_name: file_name.clone(),
        file_path: path,
        upload_url: Some(upload_url.clone().into()),
        size,
    };

    let _upload = DbUpload::create(pool, &upload_form).await?;

    Ok(upload_url)
}

pub async fn delete_emoji_file(pool: &DbPool, emoji: &Emoji) -> Result<(), TinyBoardsError> {
    // Find the upload record for this emoji
    if let Ok(upload) = DbUpload::find_by_url(pool, &emoji.image_url).await {
        // Delete the file from the file system
        if let Err(e) = std::fs::remove_file(&upload.file_path) {
            eprintln!("Failed to delete emoji file {}: {}", upload.file_path, e);
            // Continue anyway to clean up DB record
        }

        // Delete DB upload entry
        if let Err(e) = DbUpload::delete(pool, upload.id).await {
            eprintln!("Failed to delete upload record for emoji {}: {}", emoji.shortcode, e);
        }
    } else {
        eprintln!("Upload record not found for emoji: {}", emoji.shortcode);
    }

    Ok(())
}

fn is_emoji_content_type(content_type: &str) -> bool {
    matches!(
        content_type.to_lowercase().as_str(),
        "image/png" | "image/jpeg" | "image/jpg" | "image/gif" | "image/webp"
    )
}

#[allow(dead_code)]
fn sanitize_shortcode(shortcode: &str) -> String {
    shortcode
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

fn validate_emoji_dimensions(file_bytes: &[u8], _content_type: &str) -> Result<(), TinyBoardsError> {
    // Basic size check - most emoji files should be reasonable
    if file_bytes.len() < 100 {
        return Err(TinyBoardsError::from_message(
            400,
            "Emoji file appears to be corrupted or too small",
        ));
    }

    // Basic dimension validation using simple image headers
    // For production, you'd want to use the `image` crate for proper validation
    match _content_type {
        "image/png" => {
            if file_bytes.len() < 24 || !file_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                return Err(TinyBoardsError::from_message(400, "Invalid PNG file"));
            }
            // PNG width/height are at bytes 16-23 (big-endian u32s)
            if file_bytes.len() >= 24 {
                let width = u32::from_be_bytes([file_bytes[16], file_bytes[17], file_bytes[18], file_bytes[19]]);
                let height = u32::from_be_bytes([file_bytes[20], file_bytes[21], file_bytes[22], file_bytes[23]]);

                if width > MAX_EMOJI_DIMENSIONS || height > MAX_EMOJI_DIMENSIONS {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!("Emoji dimensions too large. Maximum: {}x{}, found: {}x{}",
                                MAX_EMOJI_DIMENSIONS, MAX_EMOJI_DIMENSIONS, width, height)
                    ));
                }

                if width < 8 || height < 8 {
                    return Err(TinyBoardsError::from_message(400, "Emoji too small (minimum 8x8 pixels)"));
                }
            }
        }
        "image/jpeg" | "image/jpg" => {
            if file_bytes.len() < 10 || !file_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                return Err(TinyBoardsError::from_message(400, "Invalid JPEG file"));
            }
            // For JPEG, basic validation is sufficient for emojis
        }
        "image/gif" => {
            if file_bytes.len() < 10 || !file_bytes.starts_with(&[0x47, 0x49, 0x46]) {
                return Err(TinyBoardsError::from_message(400, "Invalid GIF file"));
            }
            // GIF width/height are at bytes 6-9 (little-endian u16s)
            if file_bytes.len() >= 10 {
                let width = u16::from_le_bytes([file_bytes[6], file_bytes[7]]) as u32;
                let height = u16::from_le_bytes([file_bytes[8], file_bytes[9]]) as u32;

                if width > MAX_EMOJI_DIMENSIONS || height > MAX_EMOJI_DIMENSIONS {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!("Emoji dimensions too large. Maximum: {}x{}, found: {}x{}",
                                MAX_EMOJI_DIMENSIONS, MAX_EMOJI_DIMENSIONS, width, height)
                    ));
                }
            }
        }
        "image/webp" => {
            if file_bytes.len() < 30 || !file_bytes.starts_with(&[0x52, 0x49, 0x46, 0x46]) {
                return Err(TinyBoardsError::from_message(400, "Invalid WebP file"));
            }
            // Basic WebP validation - more complex format, but basic check is sufficient
        }
        _ => {
            // For other formats, just do basic validation
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_shortcode() {
        assert_eq!(sanitize_shortcode("test_emoji"), "test_emoji");
        assert_eq!(sanitize_shortcode("Test-Emoji123"), "test-emoji123");
        assert_eq!(sanitize_shortcode("emoji@#$%"), "emoji");
        assert_eq!(sanitize_shortcode("😀"), "");
    }

    #[test]
    fn test_is_emoji_content_type() {
        assert!(is_emoji_content_type("image/png"));
        assert!(is_emoji_content_type("image/jpeg"));
        assert!(is_emoji_content_type("IMAGE/PNG"));
        assert!(!is_emoji_content_type("application/pdf"));
        assert!(!is_emoji_content_type("text/plain"));
    }
}