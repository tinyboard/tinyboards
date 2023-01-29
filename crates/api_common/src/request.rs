use reqwest::multipart::Part;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use tinyboards_utils::{
    error::TinyBoardsError, settings::structs::Settings, version::VERSION, REQWEST_TIMEOUT,
};
use url::Url;

use crate::utils::decode_base64_image;

pub fn build_user_agent(settings: &Settings) -> String {
    format!(
        "TinyBoards/{}; +{}",
        VERSION,
        settings.get_protocol_and_hostname()
    )
}

#[tracing::instrument(skip_all)]
async fn is_image_content_type(
    client: &ClientWithMiddleware,
    url: &Url,
) -> Result<(), TinyBoardsError> {
    let response = client.get(url.as_str()).send().await?;
    if response
        .headers()
        .get("Content-Type")
        .ok_or_else(|| TinyBoardsError::from_message(400, "No Content-Type header"))?
        .to_str()?
        .starts_with("image/")
    {
        Ok(())
    } else {
        Err(TinyBoardsError::from_message(400, "Not an image type."))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsPurgeResponse {
    msg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Image {
    file: String,
    delete_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Images {
    msg: String,
    files: Option<Vec<Image>>,
    url: Option<String>,
    delete_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PictrsUploadRequest {
    pub image: Option<String>,
    pub url: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PictrsUploadResponse {
    pub url: String,
    pub delete_url: String,
}

/// Forwards image upload request to our own upload image endpoint
pub async fn upload_image_to_pictrs(
    client: &ClientWithMiddleware,
    settings: &Settings,
    image: Option<String>,
    url: Option<String>,
) -> Result<PictrsUploadResponse, TinyBoardsError> {


    if image.is_some() && url.is_some() {
        return Err(TinyBoardsError::from_message(400, "you can't input both a base64 string and a url to upload an image"))
    }

    if let Some(img_str_b64) = image {    

        let pictrs_conf = settings.pictrs_config()?;
        let image_url = format!("{}image", pictrs_conf.url);
        let (img_bytes, file_name) = decode_base64_image(img_str_b64)?;
        let img_part = Part::bytes(img_bytes).file_name(file_name);
        let form = reqwest::multipart::Form::new()
            .part("images[]", img_part);
    
        let res = client
            .post(&image_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "failed to upload image"))?;
    
        let images = res.json::<Images>().await.map_err(|e| TinyBoardsError::from_error_message(e, 500, "failed to map response to json"))?;
        
        let mut url: Option<String> = None;
        let mut delete_url: Option<String> = None;

        if let Some(files) = &images.files {
            url = Some(format!("{}/image/{}", settings.get_protocol_and_hostname(), files[0].file));
            delete_url = Some(format!("{}/image/delete/{}/{}", settings.get_protocol_and_hostname(), files[0].delete_token, files[0].file));
        }
        
        Ok(PictrsUploadResponse { url: url.unwrap(), delete_url: delete_url.unwrap() })
    } else if let Some(url) = url {
        
        let pictrs_conf = settings.pictrs_config()?;
        let image_download_url = format!("{}image/download?url={}", pictrs_conf.url, url);

        let res = client
            .get(&image_download_url)
            .send()
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "failed to upload image from URL"))?;
        
        let images = res.json::<Images>().await.map_err(|e| TinyBoardsError::from_error_message(e, 500, "failed to map response to json"))?;
        
        let mut url: Option<String> = None;
        let mut delete_url: Option<String> = None;

        if let Some(files) = &images.files {
            url = Some(format!("{}/image/{}", settings.get_protocol_and_hostname(), files[0].file));
            delete_url = Some(format!("{}/image/delete/{}/{}", settings.get_protocol_and_hostname(), files[0].delete_token, files[0].file));
        }
        
        Ok(PictrsUploadResponse { url: url.unwrap(), delete_url: delete_url.unwrap() })
    } else {
        return Err(TinyBoardsError::from_message(400, "b64 image or url not provided"));
    }
}


/// Purges image from pictrs
pub async fn purge_image_from_pictrs(
    client: &ClientWithMiddleware,
    settings: &Settings,
    image_url: &Url,
) -> Result<(), TinyBoardsError> {
    let pictrs_config = settings.pictrs_config()?;
    is_image_content_type(client, image_url).await?;

    let alias = image_url
        .path_segments()
        .ok_or_else(|| TinyBoardsError::from_message(400, "image url missing path segments"))?
        .next_back()
        .ok_or_else(|| TinyBoardsError::from_message(400, "image url missing last path segment"))?;

    let purge_url = format!("{}/internal/purge?alias={}", pictrs_config.url, alias);

    let pictrs_api_key = pictrs_config.api_key.ok_or_else(|| {
        TinyBoardsError::from_message(400, "pictrs api key not provided in settings")
    })?;

    let response = client
        .post(&purge_url)
        .timeout(REQWEST_TIMEOUT)
        .header("x-api-token", pictrs_api_key)
        .send()
        .await?;

    let response: PictrsPurgeResponse = response.json().await.map_err(TinyBoardsError::from)?;

    if response.msg == "ok" {
        Ok(())
    } else {
        Err(TinyBoardsError::from_message(400, &response.msg))
    }
}


