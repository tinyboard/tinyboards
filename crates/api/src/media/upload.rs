use std::fs::*;
use tinyboards_api_common::data::TinyBoardsContext;
use std::io::Write;
use tinyboards_utils::{error::TinyBoardsError, utils::generate_rand_string};
use actix_web::web::Data;
use tinyboards_db::{models::media::uploads::*, traits::Crud};

pub async fn handle_uploaded_image(
    context: Data<TinyBoardsContext>,
    image_data: Vec<u8>,
    file_ext: &str
) -> Result<Upload, TinyBoardsError> {


    // Save the uploaded image data to local storage with a randomized filename.
    let rand_string = generate_rand_string();
    let random_filename = format!("{}.{}",  rand_string, file_ext);
    let filepath = format!("./uploads/{}", random_filename);
    let mut file = File::create(&filepath).map_err(|_| TinyBoardsError::from_message(500, "error creating file."))?;

    file.write_all(&image_data).map_err(|_| TinyBoardsError::from_message(500, "error writing data to file."))?;

    // Add an entry into the database table `uploads` with the Diesel RS ORM.
    let upload_form = UploadForm {
        filename: random_filename,
        filepath: filepath.clone(),
    };
    let upload = Upload::create(context.pool(), &upload_form).await?;

    Ok(upload)
}