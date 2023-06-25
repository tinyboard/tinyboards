// use crate::Perform;
// use actix_files::NamedFile;
// use actix_web::{body::BodyStream, http::header::ContentType, web::Data, HttpResponse};
// use image::{DynamicImage, GenericImageView, ImageFormat};
// use std::io::Cursor;
// use std::path::PathBuf;
// use tinyboards_api_common::{
//     data::TinyBoardsContext,
//     site::{FileNamePath, GetFile},
// };
// use tinyboards_db::models::site::uploads::Upload;
// use tinyboards_utils::error::TinyBoardsError;

//#[async_trait::async_trait(?Send)]
/*impl GetFile {
    //type Response = HttpResponse;
    //type Route = FileNamePath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        _: Option<&str>,
    ) -> Result<HttpResponse, TinyBoardsError> {
        let options = &self;

        let f_name = path.file_name.clone();
        let file = Upload::find_by_name(context.pool(), &f_name).await?;

        let path = PathBuf::from("./static/media").join(&file.file_name.clone());

        if !path.exists() {
            return Err(TinyBoardsError::from_message(404, "file not found"));
        }

        if let Some(ext) = path.extension() {
            if ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "gif" || ext == "webp" {
                let image = image::open(&path).map_err(|e| {
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

        let named_file = NamedFile::open(&path)?;
        let content_type = named_file.content_type();
        let file_stream = named_file.into_stream();

        NamedFile::open(&path)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "internal server error"))
            .map(|named_file| HttpResponse::Ok().streaming(named_file))
    }
}*/
