use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
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


