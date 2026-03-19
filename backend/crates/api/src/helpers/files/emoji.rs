use crate::{DbPool, Settings};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::emoji::Emoji,
    schema::{emoji, site, uploads},
    utils::get_conn,
};
use async_graphql::*;
use std::io::Read;
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
use uuid::Uuid;

const MAX_EMOJI_DIMENSIONS: u32 = 512;

pub async fn upload_emoji_file(
    upload: Upload,
    shortcode: String,
    for_user_id: Uuid,
    ctx: &Context<'_>,
) -> Result<Url> {
    let settings = ctx.data::<Settings>()?.as_ref();
    let pool = ctx.data::<DbPool>()?;
    let media_path = settings.get_media_path();
    let conn = &mut get_conn(pool).await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    // Load site configuration for emoji file size limit
    let max_size_mb: i32 = site::table
        .select(site::emoji_max_file_size_mb)
        .first(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let mut file_bytes: Vec<u8> = Vec::new();

    let upload_value = upload.value(ctx)?;
    let content_type = upload_value.content_type.unwrap_or(String::new());
    let max_size = (max_size_mb as u32 * 1024 * 1024) as i64;

    if !is_emoji_content_type(&content_type) {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("{} is not a valid emoji file type. Only PNG, JPG, GIF, and WebP are allowed", content_type),
        )
        .into());
    }

    upload
        .value(ctx)?
        .into_read()
        .read_to_end(&mut file_bytes)?;

    if file_bytes.is_empty() {
        return Err(TinyBoardsError::from_message(
            400,
            "No file data received or file is empty",
        )
        .into());
    }

    validate_file_content(&file_bytes, &content_type).map_err(|e| {
        TinyBoardsError::from_message(400, &e)
    })?;

    let file_name = format!("emoji_{}", generate_secure_filename(Some(shortcode.clone()), &content_type));
    let path = format!("{}/emojis/{}", media_path, file_name);

    let emoji_dir = format!("{}/emojis", media_path);
    tokio::fs::create_dir_all(&emoji_dir).await?;

    let size = file_bytes.len() as i64;
    if size > max_size {
        return Err(TinyBoardsError::from_message(
            400,
            &format!("Emoji file exceeds maximum allowed size of {}", format_file_size(max_size)),
        )
        .into());
    }

    if let Err(e) = validate_emoji_dimensions(&file_bytes, &content_type) {
        return Err(e.into());
    }

    let mut file = File::create(&path).await?;
    file.write_all(&file_bytes).await?;
    file.flush().await?;

    let upload_url = Url::parse(&format!(
        "{}/media/emojis/{}",
        settings.get_protocol_and_hostname(),
        &file_name
    ))?;

    // Record upload in database
    use tinyboards_db::models::upload::UploadInsertForm;

    let upload_form = UploadInsertForm {
        user_id: for_user_id,
        original_name: format!("{}.{}", shortcode, get_file_type_extended(&content_type)),
        file_name: file_name.clone(),
        file_path: path,
        upload_url: upload_url.to_string(),
        size_bytes: size,
        thumbnail_url: None,
    };

    diesel::insert_into(uploads::table)
        .values(&upload_form)
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    Ok(upload_url)
}

pub async fn delete_emoji_file(pool: &DbPool, emoji_record: &Emoji) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool).await?;

    // Find the upload record for this emoji by URL
    let upload_result: Option<(Uuid, String)> = uploads::table
        .filter(uploads::upload_url.eq(&emoji_record.image_url))
        .select((uploads::id, uploads::file_path))
        .first(conn)
        .await
        .optional()
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    if let Some((upload_id, file_path)) = upload_result {
        if let Err(e) = std::fs::remove_file(&file_path) {
            tracing::warn!("Failed to delete emoji file {}: {}", file_path, e);
        }

        let _ = diesel::delete(uploads::table.find(upload_id))
            .execute(conn)
            .await;
    } else {
        tracing::warn!("Upload record not found for emoji: {}", emoji_record.shortcode);
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
    if file_bytes.len() < 8 {
        return Err(TinyBoardsError::from_message(
            400,
            "Emoji file appears to be corrupted or empty",
        ));
    }

    match _content_type {
        "image/png" => {
            if file_bytes.len() < 24 || !file_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                return Err(TinyBoardsError::from_message(400, "Invalid PNG file"));
            }
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
        }
        "image/gif" => {
            if file_bytes.len() < 10 || !file_bytes.starts_with(&[0x47, 0x49, 0x46]) {
                return Err(TinyBoardsError::from_message(400, "Invalid GIF file"));
            }
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
        }
        _ => {}
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
