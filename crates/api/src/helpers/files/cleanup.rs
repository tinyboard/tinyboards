use crate::{DbPool, storage::StorageBackend};
use std::collections::HashSet;
use tinyboards_db::models::site::content_uploads::{ContentUpload, ContentUploadForm};
use tinyboards_db::models::site::uploads::Upload;
use tinyboards_db::traits::Crud;
use tinyboards_utils::TinyBoardsError;
use regex::Regex;

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
    // Extract the key from the full path
    // file_path might be like "/home/user/media/images/file.jpg"
    // we need to extract just "images/file.jpg" or whatever the storage key is
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
/// This function:
/// 1. Gets all current uploads linked to the content
/// 2. Extracts image URLs from the new HTML
/// 3. Finds uploads that are no longer referenced
/// 4. Deletes orphaned files from storage
/// 5. Deletes orphaned upload records from database
pub async fn cleanup_orphaned_uploads(
    pool: &DbPool,
    content_id: i32,
    is_post: bool,  // true for post, false for comment
    new_html: &str,
    storage: &StorageBackend,
) -> Result<(), TinyBoardsError> {
    // Get current uploads for this content
    let current_uploads = if is_post {
        ContentUpload::get_by_post(pool, content_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get content uploads"))?
    } else {
        ContentUpload::get_by_comment(pool, content_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get content uploads"))?
    };

    // Extract URLs from new HTML
    let referenced_urls: HashSet<String> = extract_image_urls_from_html(new_html)
        .into_iter()
        .collect();

    // Find and delete orphaned uploads
    for content_upload in current_uploads {
        let upload = Upload::read(pool, content_upload.upload_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read upload"))?;

        let upload_url_str = upload.upload_url.to_string();

        if !referenced_urls.contains(&upload_url_str) {
            // This upload is orphaned - delete it
            tracing::info!("Deleting orphaned upload: {} ({})", upload.file_name, upload_url_str);

            // Delete from storage
            if let Err(e) = delete_from_storage(&upload.file_path, storage).await {
                tracing::error!("Failed to delete file from storage: {}", e);
                // Continue even if storage deletion fails
            }

            // Delete upload record from database
            Upload::delete(pool, upload.id)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete upload record"))?;

            // Delete content_upload junction entry
            ContentUpload::delete(pool, content_upload.id)
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete content upload link"))?;
        }
    }

    Ok(())
}

/// Delete all files associated with a post
/// Used when deleting a post
pub async fn delete_post_files(
    pool: &DbPool,
    post_id: i32,
    storage: &StorageBackend,
) -> Result<(), TinyBoardsError> {
    let content_uploads = ContentUpload::get_by_post(pool, post_id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get post uploads"))?;

    for content_upload in content_uploads {
        let upload = Upload::read(pool, content_upload.upload_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read upload"))?;

        tracing::info!("Deleting post file: {} ({})", upload.file_name, upload.upload_url);

        // Delete from storage
        if let Err(e) = delete_from_storage(&upload.file_path, storage).await {
            tracing::error!("Failed to delete file from storage: {}", e);
        }

        // Delete upload record
        Upload::delete(pool, upload.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete upload record"))?;
    }

    // Delete all content_upload entries for this post
    ContentUpload::delete_by_post(pool, post_id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete content upload links"))?;

    Ok(())
}

/// Delete all files associated with a comment
/// Used when deleting a comment
pub async fn delete_comment_files(
    pool: &DbPool,
    comment_id: i32,
    storage: &StorageBackend,
) -> Result<(), TinyBoardsError> {
    let content_uploads = ContentUpload::get_by_comment(pool, comment_id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get comment uploads"))?;

    for content_upload in content_uploads {
        let upload = Upload::read(pool, content_upload.upload_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to read upload"))?;

        tracing::info!("Deleting comment file: {} ({})", upload.file_name, upload.upload_url);

        // Delete from storage
        if let Err(e) = delete_from_storage(&upload.file_path, storage).await {
            tracing::error!("Failed to delete file from storage: {}", e);
        }

        // Delete upload record
        Upload::delete(pool, upload.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete upload record"))?;
    }

    // Delete all content_upload entries for this comment
    ContentUpload::delete_by_comment(pool, comment_id)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete content upload links"))?;

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

/// Link uploads found in HTML content to a post or comment
///
/// This function:
/// 1. Extracts all image URLs from the HTML content
/// 2. Finds matching Upload records by URL
/// 3. Creates ContentUpload records to link them
///
/// Should be called after creating a post/comment with HTML content
pub async fn link_content_uploads(
    pool: &DbPool,
    content_id: i32,
    is_post: bool,  // true for post, false for comment
    html_content: &str,
) -> Result<(), TinyBoardsError> {
    // Extract image URLs from HTML
    let image_urls = extract_image_urls_from_html(html_content);

    if image_urls.is_empty() {
        return Ok(());
    }

    // Find matching uploads and create links
    for (position, url) in image_urls.iter().enumerate() {
        // Try to find the upload by URL
        match Upload::find_by_url_str(pool, url).await {
            Ok(upload) => {
                // Create ContentUpload link
                let content_upload_form = ContentUploadForm {
                    upload_id: upload.id,
                    post_id: if is_post { Some(content_id) } else { None },
                    comment_id: if is_post { None } else { Some(content_id) },
                    position: Some(position as i32),
                };

                ContentUpload::create(pool, &content_upload_form)
                    .await
                    .map_err(|e| TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Failed to create content upload link"
                    ))?;

                tracing::info!(
                    "Linked upload {} to {} id {}",
                    upload.file_name,
                    if is_post { "post" } else { "comment" },
                    content_id
                );
            }
            Err(_) => {
                // Upload not found - could be external image or already linked
                tracing::debug!("Upload not found for URL: {}", url);
                continue;
            }
        }
    }

    Ok(())
}
