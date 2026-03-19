use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
/// Site metadata, from its opengraph tags.
pub struct SiteMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub(crate) image: Option<Url>,
    pub embed_video_url: Option<Url>,
}
