use encoding::{all::encodings, DecoderTrap};
use reqwest_middleware::ClientWithMiddleware;
use tinyboards_db::newtypes::DbUrl;
use tinyboards_utils::{
    error::TinyBoardsError, settings::structs::Settings, version::VERSION,
};
use tracing::info;
use url::Url;
use webpage::HTML;
use crate::post::SiteMetadata;

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

/// Fetches the post link html tags (like title, description, image, etc)
#[tracing::instrument(skip_all)]
pub async fn fetch_site_metadata(
  client: &ClientWithMiddleware,
  url: &Url,
) -> Result<SiteMetadata, TinyBoardsError> {
  info!("Fetching site metadata for url: {}", url);
  let response = client.get(url.as_str()).send().await?;

  let html_bytes = response.bytes().await.map_err(TinyBoardsError::from)?.to_vec();

  let tags = html_to_site_metadata(&html_bytes)?;

  Ok(tags)
}

fn html_to_site_metadata(html_bytes: &[u8]) -> Result<SiteMetadata, TinyBoardsError> {
    let html = String::from_utf8_lossy(html_bytes);
  
    // Make sure the first line is doctype html
    let first_line = html
      .trim_start()
      .lines()
      .next()
      .ok_or_else(|| TinyBoardsError::from_message(400, "no lines in html"))?
      .to_lowercase();
  
    if !first_line.starts_with("<!doctype html>") {
      return Err(TinyBoardsError::from_message(
        400, "site metadata page fetch is not DOCTYPE html",
      ));
    }
  
    let mut page = HTML::from_string(html.to_string(), None)?;
  
    // If the web page specifies that it isn't actually UTF-8, re-decode the received bytes with the
    // proper encoding. If the specified encoding cannot be found, fall back to the original UTF-8
    // version.
    if let Some(charset) = page.meta.get("charset") {
      if charset.to_lowercase() != "utf-8" {
        if let Some(encoding_ref) = encodings().iter().find(|e| e.name() == charset) {
          if let Ok(html_with_encoding) = encoding_ref.decode(html_bytes, DecoderTrap::Replace) {
            page = HTML::from_string(html_with_encoding, None)?;
          }
        }
      }
    }
  
    let page_title = page.title;
    let page_description = page.description;
  
    let og_description = page
      .opengraph
      .properties
      .get("description")
      .map(std::string::ToString::to_string);
    let og_title = page
      .opengraph
      .properties
      .get("title")
      .map(std::string::ToString::to_string);
    let og_image = page
      .opengraph
      .images
      .first()
      .and_then(|ogo| Url::parse(&ogo.url).ok());
    let og_embed_url = page
      .opengraph
      .videos
      .first()
      .and_then(|v| Url::parse(&v.url).ok());
  
    Ok(SiteMetadata {
      title: og_title.or(page_title),
      description: og_description.or(page_description),
      image: og_image.map(Into::into),
      embed_video_url: og_embed_url.map(Into::into),
    })
  }

#[tracing::instrument(skip_all)]
pub async fn fetch_site_data(
  client: &ClientWithMiddleware,
  url: Option<&Url>,
) -> (Option<SiteMetadata>, Option<DbUrl>) {
  match &url {
    Some(url) => {
      // Fetch metadata
      // Ignore errors, since it may be an image, or not have the data.
      // Warning, this may ignore SSL errors
      let metadata_option = fetch_site_metadata(client, url).await.ok().clone();

      let image_url: Option<&DbUrl> = match &metadata_option {
        Some(metadata_res) => match &metadata_res.image {
          // Metadata, with image
          // Try to generate a small thumbnail if there's a full sized one from post-links
          Some(metadata_image) => Some(metadata_image),
          // Metadata, but no image
          None => None,
        },
        // No metadata, try to fetch the URL as an image
        None => None,
      };

      let meta_option = metadata_option.clone();

      (meta_option, image_url.cloned())
    }
    None => (None, None),
  }
}