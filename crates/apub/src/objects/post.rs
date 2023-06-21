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
        post::posts::{Post, PostForm},
    },
    traits::Crud, utils::naive_now,
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
impl Object for ApubPost {
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
      Post::read_from_apub_id(context.pool(), object_id)
        .await?
        .map(Into::into)
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
    if !self.is_deleted {
      let form = PostForm { is_deleted: Some(true), updated: Some(naive_now()), ..PostForm::default() };
      Post::update(context.pool(), self.id, &form).await?;
    }
    Ok(())
  }

  /// Turn a Tinyboards post into a AP Page that can be sent over the network
  #[tracing::instrument(skip_all)]
  async fn into_json(self, context: &Data<Self::DataType>) -> Result<Page, TinyBoardsError> {
    let creator_id = self.creator_id;
    let creator = Person::read(context.pool(), creator_id).await?;
    let board_id = self.board_id;
    let board = Board::read(context.pool(), board_id).await?;
    let language = LanguageTag::new_single(self.language_id, context.pool()).await?;

    let page = Page {
      kind: PageType::Page,
      id: self.ap_id.clone().unwrap().into(),
      attributed_to: AttributedTo::TinyBoards(creator.actor_id.into()),
      to: vec![board.actor_id.clone().into(), public()],
      cc: vec![],
      name: Some(self.title.clone()),
      content: parse_markdown(&self.body),
      media_type: Some(MediaTypeMarkdownOrHtml::Html),
      source: Some(Source::new(self.body.clone())),
      attachment: self.url.clone().map(Attachment::new).into_iter().collect(),
      image: self.thumbnail_url.clone().map(ImageObject::new),
      comments_enabled: Some(!self.is_locked),
      sensitive: Some(self.is_nsfw),
      language,
      published: Some(convert_datetime(self.creation_date)),
      updated: self.updated.map(convert_datetime),
      audience: Some(board.actor_id.into()),
      in_reply_to: None,
    };

    Ok(page)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    page: &Page,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> Result<(), TinyBoardsError> {
    if !page.is_mod_action(context).await? {
      verify_domains_match(page.id.inner(), expected_domain)?;
      verify_is_remote_object(page.id.inner(), context.settings())?;
    }

    let local_site_data = fetch_local_site_data(context.pool()).await?;

    let board = page.board(context).await?;

    check_ap_id_valid_with_strictness(
      page.id.inner(), 
      board.local, 
      &local_site_data, 
      context.settings())?;
    
    verify_person_in_board(&page.creator()?, &board, context).await?;

    verify_domains_match(page.creator()?.inner(), page.id.inner())?;
    verify_is_public(&page.to, &page.cc)?;

    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(page: Page, context: &Data<Self::DataType>) -> Result<ApubPost, TinyBoardsError> {
    let creator = page.creator()?.dereference(context).await?;
    let board = page.board(context).await?;
    
    if board.posting_restricted_to_mods {
      is_mod_or_admin(context.pool(), creator.id, board.id).await?;
    }

    let mut name = page
      .name
      .clone()
      .or_else(|| {
        page
          .content
          .clone()
          .as_ref()
          .and_then(|c| parse_html(c).lines().next().map(ToString::to_string))
      })
      .ok_or_else(|| anyhow!("object must have name or content"))?;

    if name.chars().count() > MAX_TITLE_LENGTH {
      name = name.chars().take(MAX_TITLE_LENGTH).collect();
    }

    // read existing, local post if any (for generating mod log)
    let old_post = page.id.dereference_local(context).await;

    let form = if !page.is_mod_action(context).await? {
      let first_attachment = page.attachment.into_iter().map(Attachment::url).next();
      let post_url = if first_attachment.is_some() {
        first_attachment
      } else if page.kind == PageType::Video {
        // insert external video page link
        Some(page.id.inner().clone())
      } else {
        None
      };

      let (metadata_res, thumbnail_url) = match &post_url {
        Some(post_url) if old_post.is_err() => {
          fetch_site_data(context.client(), Some(post_url)).await
        },
        _ => (None, page.image.map(|i| i.url.into()))
      };

      let (_embed_title, _embed_description, _embed_video_url) = metadata_res
        .map(|u| (u.title, u.description, u.embed_video_url))
        .unwrap_or_default();

      let _local_site = LocalSite::read(context.pool()).await.ok();

      let language_id = LanguageTag::to_language_id_single(page.language, context.pool()).await?;

      PostForm {
        title: Some(name),
        url: post_url.map(Into::into),
        body: read_from_string_or_source_opt(&page.content, &page.media_type, &page.source),
        creator_id: Some(creator.id),
        board_id: Some(board.id),
        is_removed: None,
        is_locked: page.comments_enabled.map(|e| !e),
        creation_date: page.published.map(|u| u.naive_local()),
        updated: page.updated.map(|u| u.naive_local()),
        is_deleted: Some(false),
        is_nsfw: page.sensitive,
        thumbnail_url,
        ap_id: Some(page.id.clone().into()),
        local: Some(false),
        language_id,
        featured_board: None,
        featured_local: None,
        ..PostForm::default()
      }
    } else {
        PostForm {
          title: Some(name),
          creator_id: Some(creator.id),
          board_id: Some(board.id),
          ap_id: Some(page.id.clone().into()),
          is_locked: page.comments_enabled.map(|e| !e),
          updated: page.updated.map(|u| u.naive_local()),
          ..PostForm::default()
        }
    };

    let post = Post::create(context.pool(), &form).await?;

    if Page::is_locked_changed(&old_post, &page.comments_enabled) {
      let form = ModLockPostForm {
        mod_person_id: creator.id,
        post_id: post.id,
        locked: Some(Some(post.is_locked)),
      };
      ModLockPost::create(context.pool(), &form).await?;
    }

    Ok(post.into())

  }    
}