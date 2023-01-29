use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use tinyboards_utils::{
    error::TinyBoardsError, settings::structs::Settings, version::VERSION, REQWEST_TIMEOUT,
};
use url::Url;

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
    auth: &str,
    image: Option<String>,
    url: Option<String>,
) -> Result<PictrsUploadResponse, TinyBoardsError> {

    let upload_url = format!("{}/image", settings.get_protocol_and_hostname());

    let request = PictrsUploadRequest {
        image,
        url,
    };

    let auth_header = format!("Bearer {}", auth);

    let resp = client
        .post(&upload_url)
        .json(&request)
        .header("Authorization", &auth_header)
        .send()
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "failed to upload image"))?;

    
    let images = resp.json::<Images>().await.map_err(|_e| TinyBoardsError::from_message(500, "failed mapping response into json object")).unwrap();

    Ok(PictrsUploadResponse { url: images.url.unwrap(), delete_url: images.delete_url.unwrap() })
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


