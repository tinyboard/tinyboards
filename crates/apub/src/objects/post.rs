use crate::{
    activities::{verify_is_public, verify_person_in_board},
    check_ap_id_valid_with_strictness,
    fetch_local_site_data,
    objects::{read_from_string_or_source_opt, verify_is_remote_object},
    protocol::{
      objects::{
        page::{Attachment, AttributedTo, Page, PageType},
        LanguageTag,
      },
      ImageObject,
      InBoard,
      Source,
    },
  };
  use tinyboards_federation::{
    config::Data,
    kinds::public,
    protocol::{values::MediaTypeMarkdownOrHtml, verification::verify_domains_match},
    traits::Object,
  };
  use anyhow::anyhow;
  use chrono::NaiveDateTime;
  use html2md::parse_html;
  use tinyboards_api_common::{
    data::TinyBoardsContext,
    request::fetch_site_data,
    utils::{is_mod_or_admin},
  };
  use tinyboards_db::{
    self,
    models::{
        board::boards::Board,
        site::local_site::LocalSite,
        moderator::mod_actions::{ModLockPost, ModLockPostForm},
        person::person::Person,
        post::posts::*,
    },
    traits::Crud,
  };
  use tinyboards_utils::{
    error::TinyBoardsError,
    parser::parse_markdown,
    time::convert_datetime,
  };
  use std::ops::Deref;
  use url::Url;
  
  const MAX_TITLE_LENGTH: usize = 200;

  #[derive(Clone, Debug)]
pub struct ApubPost(pub(crate) Post);

impl Deref for ApubPost {
  type Target = Post;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Post> for ApubPost {
  fn from(p: Post) -> Self {
    ApubPost(p)
  }
}

#[async_trait::async_trait]
impl Object for Post {
  type DataType = TinyBoardsContext;
  type Kind = Page;
  type Error = TinyBoardsError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> Result<Option<Self>, TinyBoardsError> {
    Ok(
      Post::r
    )
  }
}