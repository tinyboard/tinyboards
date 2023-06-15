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