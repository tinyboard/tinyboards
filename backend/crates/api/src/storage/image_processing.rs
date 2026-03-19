use image::{ImageFormat, ImageReader, DynamicImage};
use std::io::Cursor;
use tinyboards_utils::error::TinyBoardsError;

/// Settings that control how uploaded images are processed.
pub struct ImageProcessingSettings {
    /// Maximum width in pixels before the image is resized down.
    pub max_width: u32,
    /// Maximum height in pixels before the image is resized down.
    pub max_height: u32,
    /// Width of the generated thumbnail (height is calculated from aspect ratio).
    pub thumbnail_width: u32,
    /// Convert JPEG and PNG to WebP for smaller file sizes. GIF is never converted
    /// because the `image` crate does not support animated WebP encoding.
    pub convert_to_webp: bool,
    /// Strip EXIF metadata from JPEG images by re-encoding. PNG and WebP do not
    /// carry meaningful EXIF data and are re-encoded regardless.
    pub strip_exif: bool,
}

impl Default for ImageProcessingSettings {
    fn default() -> Self {
        Self {
            max_width: 2048,
            max_height: 2048,
            thumbnail_width: 300,
            convert_to_webp: true,
            strip_exif: true,
        }
    }
}

/// The result of processing an uploaded image.
pub struct ProcessedImage {
    /// Processed image bytes (resized, possibly converted to WebP).
    pub data: Vec<u8>,
    /// MIME type of the processed image (may differ from input if converted).
    pub mime_type: String,
    /// Thumbnail bytes, if the image was large enough to warrant one.
    pub thumbnail_data: Option<Vec<u8>>,
    /// Final width in pixels.
    pub width: u32,
    /// Final height in pixels.
    pub height: u32,
    /// Size of the original upload in bytes.
    pub original_size: usize,
    /// Size of the processed output in bytes.
    pub processed_size: usize,
}

/// Validate and process an uploaded image: resize, strip EXIF, convert format,
/// and generate a thumbnail.
///
/// Only `image/jpeg`, `image/png`, `image/gif`, and `image/webp` are accepted.
/// GIF images are passed through without conversion to preserve animation.
pub fn process_upload(
    data: &[u8],
    mime_type: &str,
    settings: &ImageProcessingSettings,
) -> Result<ProcessedImage, TinyBoardsError> {
    // Validate MIME type
    let format = match mime_type {
        "image/jpeg" => ImageFormat::Jpeg,
        "image/png" => ImageFormat::Png,
        "image/gif" => ImageFormat::Gif,
        "image/webp" => ImageFormat::WebP,
        _ => {
            return Err(TinyBoardsError::from_message(
                400,
                &format!("Unsupported image type: {}. Accepted: jpeg, png, gif, webp", mime_type),
            ));
        }
    };

    let original_size = data.len();

    // GIF pass-through: we don't process GIFs because the image crate
    // loses animation frames. Return the original data with a thumbnail
    // generated from the first frame.
    if format == ImageFormat::Gif {
        let reader = ImageReader::with_format(Cursor::new(data), ImageFormat::Gif);
        let img = reader.decode().map_err(|e| {
            TinyBoardsError::from_message(400, &format!("Failed to decode GIF: {}", e))
        })?;

        let (w, h) = (img.width(), img.height());
        let thumbnail_data = generate_thumbnail(&img, settings.thumbnail_width)?;

        return Ok(ProcessedImage {
            data: data.to_vec(),
            mime_type: "image/gif".to_string(),
            thumbnail_data: Some(thumbnail_data),
            width: w,
            height: h,
            original_size,
            processed_size: original_size,
        });
    }

    // Decode the image
    let reader = ImageReader::with_format(Cursor::new(data), format);
    let mut img = reader.decode().map_err(|e| {
        TinyBoardsError::from_message(400, &format!("Failed to decode image: {}", e))
    })?;

    // Resize if dimensions exceed limits (preserves aspect ratio)
    if img.width() > settings.max_width || img.height() > settings.max_height {
        img = img.resize(
            settings.max_width,
            settings.max_height,
            image::imageops::FilterType::Lanczos3,
        );
    }

    let (width, height) = (img.width(), img.height());

    // Generate thumbnail
    let thumbnail_data = generate_thumbnail(&img, settings.thumbnail_width)?;

    // Determine output format and encode
    let should_convert_to_webp = settings.convert_to_webp
        && matches!(format, ImageFormat::Jpeg | ImageFormat::Png);

    let (output_data, output_mime) = if should_convert_to_webp {
        encode_image(&img, ImageFormat::WebP)?
    } else {
        // Re-encode in the original format (this strips EXIF for JPEG
        // since the image crate does not preserve metadata on re-encode)
        encode_image(&img, format)?
    };

    let processed_size = output_data.len();

    Ok(ProcessedImage {
        data: output_data,
        mime_type: output_mime,
        thumbnail_data: Some(thumbnail_data),
        width,
        height,
        original_size,
        processed_size,
    })
}

/// Resize the image to the given width (preserving aspect ratio) and encode
/// as WebP for small file size.
fn generate_thumbnail(
    img: &DynamicImage,
    thumb_width: u32,
) -> Result<Vec<u8>, TinyBoardsError> {
    // Only generate thumbnail if the image is wider than the target
    let thumb = if img.width() > thumb_width {
        img.resize(
            thumb_width,
            u32::MAX, // height calculated from aspect ratio
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img.clone()
    };

    let mut buf = Cursor::new(Vec::new());
    thumb.write_to(&mut buf, ImageFormat::WebP).map_err(|e| {
        TinyBoardsError::from_message(500, &format!("Failed to encode thumbnail: {}", e))
    })?;

    Ok(buf.into_inner())
}

/// Encode a DynamicImage to the specified format, returning the bytes and MIME type.
fn encode_image(
    img: &DynamicImage,
    format: ImageFormat,
) -> Result<(Vec<u8>, String), TinyBoardsError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, format).map_err(|e| {
        TinyBoardsError::from_message(500, &format!("Failed to encode image: {}", e))
    })?;

    let mime = match format {
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Png => "image/png",
        ImageFormat::WebP => "image/webp",
        ImageFormat::Gif => "image/gif",
        _ => "application/octet-stream",
    };

    Ok((buf.into_inner(), mime.to_string()))
}
