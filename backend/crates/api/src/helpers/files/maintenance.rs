use crate::DbPool;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::collections::HashSet;
use std::path::Path;
use tinyboards_db::{
    models::upload::Upload as DbUpload,
    schema::uploads,
    utils::get_conn,
};
use tinyboards_utils::TinyBoardsError;
use tokio::fs;
use uuid::Uuid;

/// Clean up orphaned files that exist on disk but not in database
#[allow(dead_code)]
pub async fn cleanup_orphaned_files(pool: &DbPool, media_path: &str) -> Result<usize, TinyBoardsError> {
    let mut deleted_count = 0;

    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    // Get all upload records from database
    let db_uploads: Vec<DbUpload> = uploads::table
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
    let db_file_paths: HashSet<String> = db_uploads.into_iter().map(|upload| upload.file_path).collect();

    // Scan directories for files
    let directories = [
        media_path,
        &format!("{}/emojis", media_path),
        &format!("{}/avatars", media_path),
        &format!("{}/videos", media_path),
        &format!("{}/audio", media_path),
        &format!("{}/documents", media_path),
    ];

    for directory in directories {
        if let Ok(mut entries) = fs::read_dir(directory).await {
            while let Some(entry) = entries.next_entry().await.map_err(|e| TinyBoardsError::Database(e.to_string()))? {
                let path = entry.path();
                if path.is_file() {
                    let file_path = path.to_string_lossy().to_string();

                    // Skip certain system files
                    if let Some(filename) = path.file_name() {
                        let filename_str = filename.to_string_lossy();
                        if filename_str.starts_with('.') || filename_str == "default_pfp.png" {
                            continue;
                        }
                    }

                    // If file is not tracked in database, delete it
                    if !db_file_paths.contains(&file_path) {
                        if let Err(e) = fs::remove_file(&path).await {
                            tracing::error!("Failed to delete orphaned file {}: {}", file_path, e);
                        } else {
                            tracing::info!("Deleted orphaned file: {}", file_path);
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}

/// Find database entries where files don't exist on disk
#[allow(dead_code)]
pub async fn find_broken_upload_records(pool: &DbPool) -> Result<Vec<(Uuid, String)>, TinyBoardsError> {
    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let uploads: Vec<DbUpload> = uploads::table
        .load(conn)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    let mut broken_records = Vec::new();

    for upload in uploads {
        if !Path::new(&upload.file_path).exists() {
            broken_records.push((upload.id, upload.file_path.clone()));
        }
    }

    Ok(broken_records)
}

/// Remove database entries for files that don't exist on disk
#[allow(dead_code)]
pub async fn cleanup_broken_upload_records(pool: &DbPool) -> Result<usize, TinyBoardsError> {
    let broken_records = find_broken_upload_records(pool).await?;
    let count = broken_records.len();

    let conn = &mut get_conn(pool)
        .await
        .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

    for (upload_id, file_path) in broken_records {
        if let Err(e) = diesel::delete(uploads::table.find(upload_id))
            .execute(conn)
            .await
        {
            tracing::error!("Failed to delete broken upload record {}: {}", file_path, e);
        } else {
            tracing::info!("Cleaned up broken upload record: {}", file_path);
        }
    }

    Ok(count)
}

/// Get storage statistics for the media directory
#[allow(dead_code)]
pub async fn get_storage_stats(media_path: &str) -> Result<StorageStats, TinyBoardsError> {
    let mut stats = StorageStats::default();

    let directories = [
        ("root", media_path),
        ("emojis", &format!("{}/emojis", media_path)),
        ("avatars", &format!("{}/avatars", media_path)),
        ("videos", &format!("{}/videos", media_path)),
        ("audio", &format!("{}/audio", media_path)),
        ("documents", &format!("{}/documents", media_path)),
    ];

    for (category, directory) in directories {
        if let Ok(mut entries) = fs::read_dir(directory).await {
            while let Some(entry) = entries.next_entry().await.map_err(|e| TinyBoardsError::Database(e.to_string()))? {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = entry.metadata().await {
                        let size = metadata.len();
                        stats.total_size += size;
                        stats.total_files += 1;

                        match category {
                            "emojis" => {
                                stats.emoji_size += size;
                                stats.emoji_count += 1;
                            }
                            "avatars" => {
                                stats.avatar_size += size;
                                stats.avatar_count += 1;
                            }
                            "videos" => {
                                stats.video_size += size;
                                stats.video_count += 1;
                            }
                            "audio" => {
                                stats.audio_size += size;
                                stats.audio_count += 1;
                            }
                            "documents" => {
                                stats.document_size += size;
                                stats.document_count += 1;
                            }
                            _ => {
                                stats.other_size += size;
                                stats.other_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(stats)
}

/// Storage statistics structure
#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct StorageStats {
    pub total_size: u64,
    pub total_files: u64,
    pub emoji_size: u64,
    pub emoji_count: u64,
    pub avatar_size: u64,
    pub avatar_count: u64,
    pub video_size: u64,
    pub video_count: u64,
    pub audio_size: u64,
    pub audio_count: u64,
    pub document_size: u64,
    pub document_count: u64,
    pub other_size: u64,
    pub other_count: u64,
}

impl StorageStats {
    #[allow(dead_code)]
    pub fn format_summary(&self) -> String {
        use tinyboards_utils::utils::format_file_size;

        format!(
            "Storage Summary:\n\
            Total: {} files, {}\n\
            Emojis: {} files, {}\n\
            Avatars: {} files, {}\n\
            Videos: {} files, {}\n\
            Audio: {} files, {}\n\
            Documents: {} files, {}\n\
            Other: {} files, {}",
            self.total_files, format_file_size(self.total_size as i64),
            self.emoji_count, format_file_size(self.emoji_size as i64),
            self.avatar_count, format_file_size(self.avatar_size as i64),
            self.video_count, format_file_size(self.video_size as i64),
            self.audio_count, format_file_size(self.audio_size as i64),
            self.document_count, format_file_size(self.document_size as i64),
            self.other_count, format_file_size(self.other_size as i64),
        )
    }
}

/// Delete old temporary files
#[allow(dead_code)]
pub async fn cleanup_temp_files(media_path: &str, max_age_hours: u64) -> Result<usize, TinyBoardsError> {
    let temp_dir = format!("{}/temp", media_path);
    let mut deleted_count = 0;

    if let Ok(mut entries) = fs::read_dir(&temp_dir).await {
        let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_hours * 3600);

        while let Some(entry) = entries.next_entry().await.map_err(|e| TinyBoardsError::Database(e.to_string()))? {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(created) = metadata.created().or_else(|_| metadata.modified()) {
                        if created < cutoff {
                            if let Err(e) = fs::remove_file(&path).await {
                                tracing::error!("Failed to delete temp file {}: {}", path.display(), e);
                            } else {
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}
