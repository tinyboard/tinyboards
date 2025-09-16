use actix_files::NamedFile;
use actix_web::{web::{Data, Path, Query}, HttpRequest, HttpResponse};
use image::{DynamicImage, GenericImageView, ImageFormat};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::PathBuf;
use tinyboards_db::{models::site::uploads::Upload, newtypes::DbUrl};
use tinyboards_utils::TinyBoardsError;

use crate::context::TinyBoardsContext;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUploadResponse {
    pub uploads: Vec<DbUrl>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetFile {
    pub thumbnail: Option<u32>,
    pub blur: Option<f32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteFile {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileNamePath {
    pub file_name: String,
}

impl GetFile {
    pub async fn perform(
        data: Query<Self>,
        context: Data<TinyBoardsContext>,
        path: Path<String>,
        req: HttpRequest,
    ) -> Result<HttpResponse, TinyBoardsError> {
        let options = &data.into_inner();
        let file_name = path.into_inner();

        // default pfp is assumed to be always present
        let media_path = &context.settings().get_media_path();
        let file_path = match Upload::find_by_name(context.pool(), &file_name).await {
            Ok(file) => format!("{}/{}", media_path, file.file_name),
            Err(_) => "config/file_not_found.jpg".to_string(),
        };

        let fs_path = PathBuf::from(&file_path);

        if !fs_path.exists() {
            return Err(TinyBoardsError::from_message(404, "file not found"));
        }

        if let Some(ext) = fs_path.extension() {
            // gif is not included here because it just needs to be opened, no image manipulation should apply
            if ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "webp" {
                let image = image::open(&fs_path).map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "internal server error")
                })?;

                let mut image = DynamicImage::ImageRgba8(image.to_rgba8());

                if let Some(thumbnail_size) = options.thumbnail {
                    image = image.thumbnail(thumbnail_size, thumbnail_size);
                }

                if let Some(blur_value) = options.blur {
                    image = image.blur(blur_value);
                }

                if let Some(width) = options.width {
                    if let Some(height) = options.height {
                        image = image.resize(width, height, image::imageops::FilterType::Lanczos3);
                    } else {
                        image = image.resize(
                            width,
                            image.height(),
                            image::imageops::FilterType::Lanczos3,
                        );
                    }
                }

                let mut buffer = Cursor::new(Vec::new());
                image.write_to(&mut buffer, ImageFormat::Png).map_err(|e| {
                    TinyBoardsError::from_error_message(e, 500, "internal server error")
                })?;

                return Ok(HttpResponse::Ok().body(buffer.into_inner()));
            }
        }

        NamedFile::open(&fs_path)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "internal server error"))
            .map(|named_file| named_file.into_response(&req))
    }
}