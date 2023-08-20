pub mod admin;
pub mod applications;
pub mod board;
pub mod build_response;
pub mod comment;
pub mod emoji;
pub mod data;
pub mod moderator;
pub mod person;
pub mod post;
pub mod request;
pub mod sensitive;
pub mod site;
pub mod utils;
pub mod websocket;

use crate::data::TinyBoardsContext;
use actix_files::NamedFile;
use actix_web::{
    web::{Data, Path, Query},
    HttpRequest,
    HttpResponse,
};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;
use std::path::PathBuf;
use tinyboards_db::models::site::uploads::Upload;
use tinyboards_utils::TinyBoardsError;
use crate::site::GetFile;

impl GetFile {
    pub async fn perform(
        data: Query<Self>,
        context: Data<TinyBoardsContext>,
        path: Path<String>,
        req: HttpRequest,
    ) -> Result<HttpResponse, TinyBoardsError> {
        let options = &data.into_inner();

        let f_name = path.into_inner();
        let file = Upload::find_by_name(context.pool(), &f_name).await?;

        let media_path = &context.settings().get_media_path();
        let file_name = &file.file_name;
        let file_path = &format!("./{}/{}", media_path, file_name);
        let fs_path = PathBuf::from(file_path);

        if !fs_path.exists() {
            return Err(TinyBoardsError::from_message(404, "file not found"));
        }

        if let Some(ext) = fs_path.extension() {
            if ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "gif" || ext == "webp" {
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
