use crate::TinyBoardsContext;
use actix_web::web::Data;
use rustube::{Id, VideoFetcher};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tinyboards_db::models::site::uploads::*;
use tinyboards_db::newtypes::DbUrl;
use tinyboards_db::traits::Crud;
use tinyboards_utils::error::TinyBoardsError;
use url::Url;

pub async fn save_audio_from_yt_link(
    video_link: String,
    user_id: i32,
    context: &Data<TinyBoardsContext>,
) -> Result<DbUrl, TinyBoardsError> {
    let id = Id::from_raw(&video_link)
        .map_err(|e| TinyBoardsError::from_error_message(e, 400, "Invalid YouTube link."))?;
    let descrambler = VideoFetcher::from_id(id.into_owned())?
        .fetch()
        .await
        .map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Server error while fetching YT video.")
        })?;

    let video_details = descrambler.video_details();
    // max allowed video length is 10 minutes
    if video_details.length_seconds > 600 {
        return Err(TinyBoardsError::from_message(
            413,
            "Video too long! Max allowed length is 10 minutes.",
        ));
    }

    let video = descrambler.descramble().map_err(|e| {
        TinyBoardsError::from_error_message(
            e,
            500,
            "Well, this is awkward. Decoding your video has failed.",
        )
    })?;

    if video.is_age_restricted() {
        return Err(TinyBoardsError::from_message(
            403,
            "That video is age-restricted so it cannot be downloaded. Sorry.",
        ));
    }

    let file_name = format!("profile_music_{}.mp3", user_id);

    let media_path = &context.settings().get_media_path();
    let protocol_and_hostname = &context.settings().get_protocol_and_hostname();

    let destination_path_string = format!("{}/{}", media_path, &file_name);
    let destination_path = Path::new(&destination_path_string);
    let upload_url = format!("{}/media/{}", protocol_and_hostname, &file_name);

    let best_audio_stream = video
        .streams()
        .iter()
        .filter(|stream| stream.includes_audio_track && !stream.includes_video_track)
        .max_by_key(|stream| stream.quality_label);

    match best_audio_stream {
        Some(stream) => {
            stream.download_to(destination_path).await.map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Downloading video failed.")
            })?;
        }
        None => {
            return Err(TinyBoardsError::from_message(
                500,
                "No available audio-only stream for the given video.",
            ));
        }
    }

    let mut f = File::open(destination_path).map_err(|e| {
        TinyBoardsError::from_error_message(e, 500, "Another server error? Really?")
    })?;

    let file_bytes: Vec<u8> = Vec::new();
    f.write_all(&file_bytes)?;

    let upload_form = UploadForm {
        person_id: user_id,
        original_name: file_name.clone(),
        file_name: file_name,
        file_path: destination_path.to_str().map(|p| String::from(p)).unwrap(),
        upload_url: Some(Url::parse(&upload_url)?.into()),
        size: file_bytes.len().try_into().unwrap(),
    };

    let upload = Upload::create(context.pool(), &upload_form).await?;

    Ok(upload.upload_url)
}
