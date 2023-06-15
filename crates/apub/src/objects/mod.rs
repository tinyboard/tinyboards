use crate::protocol::Source;
use tinyboards_federation::protocol::values::MediaTypeMarkdownOrHtml;
use anyhow::anyhow;
use html2md::parse_html;
use tinyboards_utils::{error::TinyBoardsError, settings::structs::Settings};
use url::Url;

pub mod person;
pub mod instance;
pub mod post;
pub mod board;

pub(crate) fn read_from_string_or_source(
    content: &str,
    media_type: &Option<MediaTypeMarkdownOrHtml>,
    source: &Option<Source>,
) -> String {
    if let Some(s) = source {
        // md sent by tinyboards in the source field
        s.content.clone()
    } else if media_type == &Some(MediaTypeMarkdownOrHtml::Markdown) {
        // md sent by things like peertube in content field
        content.to_string()
    } else {
        parse_html(content)
    }
}

pub(crate) fn read_from_string_or_source_opt(
    content: &Option<String>,
    media_type: &Option<MediaTypeMarkdownOrHtml>,
    source: &Option<Source>,
  ) -> Option<String> {
    content
      .as_ref()
      .map(|content| read_from_string_or_source(content, media_type, source))
}

/// When for example a Post is made in a remote board, the board will send it back,
/// wrapped in Announce. If it is simply received like any other federated object, overwrite the
/// existing, local Post. In particular, it will set the field local = false, so that the object
/// can't be fetched from the Activitypub HTTP endpoint anymore (which only serves local objects).
pub(crate) fn verify_is_remote_object(id: &Url, settings: &Settings) -> Result<(), TinyBoardsError> {
    let local_domain = settings.get_hostname_without_port()?;
    if id.domain() == Some(&local_domain) {
      Err(anyhow!("can't accept local object from remote instance").into())
    } else {
      Ok(())
    }
}

