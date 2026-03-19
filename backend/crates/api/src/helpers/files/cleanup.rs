use crate::{DbPool, storage::StorageBackend};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashSet;
use tinyboards_db::{
    models::upload::{ContentUpload, ContentUploadInsertForm, Upload},
    schema::{content_uploads, uploads},
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use regex::Regex;
use uuid::Uuid;

/// Extract all image URLs from HTML content
/// Finds all <img src="..."> tags and returns the src attribute values
pub fn extract_image_urls_from_html(html: &str) -> Vec<String> {
    let re = Regex::new(r#"<img[^>]+src="([^"]+)""#).unwrap();
    re.captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Delete a file from storage
/// Extracts the storage key from the file path and deletes via StorageBackend
pub async fn delete_from_storage(
    file_path: &str,
    storage: &StorageBackend,
) -> Result<(), String> {
    let key = file_path
        .split("/media/")
        .last()
        .unwrap_or(file_path);

    storage.delete(key)
        .await
        .map_err(|e| format!("Failed to delete file: {}", e))
}

/// Clean up orphaned uploads when editing post/comment content
///
/// 1. Gets all current uploads linked to the content
/// 2. Extracts image URLs from the new HTML
/// 3. Finds uploads that are no longer referenced
/// 4. Deletes orphaned files from storage
/// 5. Deletes orphaned upload records from database
pub async fn cleanup_orphaned_uploads(
    pool: &DbPool,
    content_id: Uuid,
    is_post: bool,
    new_html: &str,
    storage: &StorageBackend,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    // Get current uploads for this content
    let current_uploads: Vec<ContentUpload> = if is_post {
        content_uploads::table
            .filter(content_uploads::post_id.eq(content_id))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
    } else {
        content_uploads::table
            .filter(content_uploads::comment_id.eq(content_id))
            .load(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?
    };

    // Extract URLs from new HTML
    let referenced_urls: HashSet<String> = extract_image_urls_from_html(new_html)
        .into_iter()
        .collect();

    // Find and delete orphaned uploads
    for content_upload in current_uploads {
        let upload: Upload = uploads::table
            .find(content_upload.upload_id)
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        let upload_url_str = upload.upload_url.to_string();

        if !referenced_urls.contains(&upload_url_str) {
            tracing::info!("Deleting orphaned upload: {} ({})", upload.file_name, upload_url_str);

            if let Err(e) = delete_from_storage(&upload.file_path, storage).await {
                tracing::error!("Failed to delete file from storage: {}", e);
            }

            // Delete upload record
            diesel::delete(uploads::table.find(upload.id))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            // Delete content_upload junction entry
            diesel::delete(content_uploads::table.find(content_upload.id))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }
    }

    Ok(())
}

/// Delete all files associated with a post
pub async fn delete_post_files(
    pool: &DbPool,
    post_id: Uuid,
    storage: &StorageBackend,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let content_uploads_list: Vec<ContentUpload> = content_uploads::table
        .filter(content_uploads::post_id.eq(post_id))
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    for content_upload in content_uploads_list {
        let upload: Upload = uploads::table
            .find(content_upload.upload_id)
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        tracing::info!("Deleting post file: {} ({})", upload.file_name, upload.upload_url);

        if let Err(e) = delete_from_storage(&upload.file_path, storage).await {
            tracing::error!("Failed to delete file from storage: {}", e);
        }

        diesel::delete(uploads::table.find(upload.id))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
    }

    // Delete all content_upload entries for this post
    diesel::delete(content_uploads::table.filter(content_uploads::post_id.eq(post_id)))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    Ok(())
}

/// Delete all files associated with a comment
pub async fn delete_comment_files(
    pool: &DbPool,
    comment_id: Uuid,
    storage: &StorageBackend,
) -> Result<(), TinyBoardsError> {
    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let content_uploads_list: Vec<ContentUpload> = content_uploads::table
        .filter(content_uploads::comment_id.eq(comment_id))
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    for content_upload in content_uploads_list {
        let upload: Upload = uploads::table
            .find(content_upload.upload_id)
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        tracing::info!("Deleting comment file: {} ({})", upload.file_name, upload.upload_url);

        if let Err(e) = delete_from_storage(&upload.file_path, storage).await {
            tracing::error!("Failed to delete file from storage: {}", e);
        }

        diesel::delete(uploads::table.find(upload.id))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
    }

    diesel::delete(content_uploads::table.filter(content_uploads::comment_id.eq(comment_id)))
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    Ok(())
}

/// Link uploads found in HTML content to a post or comment
///
/// Extracts image URLs from HTML, finds matching Upload records, and creates
/// ContentUpload records to link them. Call after creating a post/comment.
pub async fn link_content_uploads(
    pool: &DbPool,
    content_id: Uuid,
    is_post: bool,
    html_content: &str,
) -> Result<(), TinyBoardsError> {
    let image_urls = extract_image_urls_from_html(html_content);

    if image_urls.is_empty() {
        return Ok(());
    }

    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    for (position, url) in image_urls.iter().enumerate() {
        // Try to find the upload by URL
        let upload_result: Result<Upload, _> = uploads::table
            .filter(uploads::upload_url.eq(url))
            .first(conn)
            .await;

        match upload_result {
            Ok(upload) => {
                let form = ContentUploadInsertForm {
                    upload_id: upload.id,
                    post_id: if is_post { Some(content_id) } else { None },
                    comment_id: if is_post { None } else { Some(content_id) },
                    position: Some(position as i32),
                };

                diesel::insert_into(content_uploads::table)
                    .values(&form)
                    .execute(conn)
                    .await
                    .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

                tracing::info!(
                    "Linked upload {} to {} id {}",
                    upload.file_name,
                    if is_post { "post" } else { "comment" },
                    content_id
                );
            }
            Err(_) => {
                tracing::debug!("Upload not found for URL: {}", url);
                continue;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_image_urls() {
        let html = r#"
            <p>Here's some text</p>
            <img src="https://example.com/image1.jpg" alt="Image 1">
            <p>More text</p>
            <img src="https://example.com/image2.png" alt="Image 2" />
            <img src="/media/uploads/image3.gif">
        "#;

        let urls = extract_image_urls_from_html(html);
        assert_eq!(urls.len(), 3);
        assert!(urls.contains(&"https://example.com/image1.jpg".to_string()));
        assert!(urls.contains(&"https://example.com/image2.png".to_string()));
        assert!(urls.contains(&"/media/uploads/image3.gif".to_string()));
    }

    #[test]
    fn test_extract_no_images() {
        let html = "<p>Just some text with no images</p>";
        let urls = extract_image_urls_from_html(html);
        assert_eq!(urls.len(), 0);
    }
}
