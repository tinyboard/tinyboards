use crate::{
    activities::{verify_is_public, verify_person_in_board},
    check_ap_id_valid_with_strictness,
    fetch_local_site_data,
    mentions::collect_non_local_mentions,
    objects::{read_from_string_or_source, verify_is_remote_object},
    protocol::{
      objects::{note::Note, LanguageTag},
      InBoard,
      Source,
    }, fetcher::post_or_comment::PostOrComment,
  };
  use tinyboards_federation::{
    config::Data,
    kinds::{object::NoteType, public},
    protocol::{values::MediaTypeMarkdownOrHtml, verification::verify_domains_match},
    traits::Object, fetch::object_id::ObjectId,
  };
  use chrono::NaiveDateTime;
  use tinyboards_api_common::data::TinyBoardsContext;
  use tinyboards_db::{
    models::{
        comment::comments::{Comment, CommentForm},
        board::boards::Board,
        site::local_site::LocalSite,
        person::person::Person,
        post::posts::Post
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
  
  #[derive(Clone, Debug)]
  pub struct ApubComment(pub(crate) Comment);
  
  impl Deref for ApubComment {
    type Target = Comment;
    fn deref(&self) -> &Self::Target {
      &self.0
    }
  }
  
  impl From<Comment> for ApubComment {
    fn from(c: Comment) -> Self {
      ApubComment(c)
    }
  }
  
  #[async_trait::async_trait]
  impl Object for ApubComment {
    type DataType = TinyBoardsContext;
    type Kind = Note;
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
            Comment::read_from_apub_id(context.pool(), object_id)
            .await?
            .map(Into::into)
        )
    }

    #[tracing::instrument(skip_all)]
    async fn delete(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
        if !self.is_deleted {
            let form = CommentForm { is_deleted: Some(true), updated: Some(naive_now()), ..CommentForm::default() };
            Comment::update(context.pool(), self.id, &form).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn into_json(self, context: &Data<Self::DataType>) -> Result<Note, TinyBoardsError> {
        let creator_id = self.creator_id;
        let creator = Person::read(context.pool(), creator_id).await?;
        let post_id = self.post_id;
        let post = Post::read(context.pool(), post_id).await?;
        let board_id = self.board_id;
        let board = Board::read(context.pool(), board_id).await?;
        
        let in_reply_to: ObjectId<PostOrComment> = if let Some(comment_id) = self.parent_comment_id() {
            let parent_comment = Comment::read(context.pool(), comment_id).await?;
            parent_comment.ap_id.unwrap().into()
        } else {
            post.ap_id.unwrap().into()
        };

        let language = LanguageTag::new_single(self.language_id, context.pool()).await?;
        let maa = collect_non_local_mentions(&self, board.actor_id.clone().into(), context).await?;

        let note = Note {
            r#type: NoteType::Note,
            id: self.ap_id.clone().unwrap().into(),
            attributed_to: creator.actor_id.into(),
            to: vec![public()],
            cc: maa.ccs,
            content: parse_markdown(&self.body).unwrap(),
            media_type: Some(MediaTypeMarkdownOrHtml::Html),
            source: Some(Source::new(self.body.clone())),
            in_reply_to,
            published: Some(convert_datetime(self.creation_date)),
            updated: self.updated.map(convert_datetime),
            tag: maa.tags,
            language,
            audience: Some(board.actor_id.into()),
        };

        Ok(note)
    }

    #[tracing::instrument(skip_all)]
    async fn verify(
        note: &Note,
        expected_domain: &Url,
        context: &Data<TinyBoardsContext>
    ) -> Result<(), TinyBoardsError> {
        verify_domains_match(note.id.inner(), expected_domain)?;
        verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
        verify_is_public(&note.to, &note.cc)?;
        let board = note.board(context).await?;
        let local_site_data = fetch_local_site_data(context.pool()).await?;

        check_ap_id_valid_with_strictness(
            note.id.inner(), 
            board.local, 
            &local_site_data, 
            context.settings(),
        )?;

        verify_is_remote_object(note.id.inner(), context.settings())?;
        verify_person_in_board(&note.attributed_to, &board, context).await?;
        let (post, _) = note.get_parents(context).await?;
        if post.is_locked {
            return Err(TinyBoardsError::from_message(400, "post is locked"));
        }
        Ok(())
    }
    
    #[tracing::instrument(skip_all)]
    async fn from_json(note: Note, context: &Data<TinyBoardsContext>) -> Result<ApubComment, TinyBoardsError> {
        let creator = note.attributed_to.dereference(context).await?;
        let (post, parent_comment) = note.get_parents(context).await?;
        let parent_id = parent_comment.map(|p| p.id);

        let body = Some(read_from_string_or_source(&note.content, &note.media_type, &note.source));
        let body_html = parse_markdown(&read_from_string_or_source(&note.content, &note.media_type, &note.source));

        let _local_site = LocalSite::read(context.pool()).await.ok();
        let language_id = LanguageTag::to_language_id_single(note.language.clone(), context.pool()).await?;
        let board = note.board(context).await?;

        let form = CommentForm {
            creator_id: creator.id,
            post_id: post.id,
            parent_id,
            body,
            body_html,
            is_removed: None,
            creation_date: note.published.map(|u| u.naive_local()),
            updated: note.updated.map(|u| u.naive_local()),
            is_deleted: Some(false),
            ap_id: Some(note.id.into()),
            local: Some(false),
            language_id,
            board_id: Some(board.id),
            ..CommentForm::default()
        };

        let comment = Comment::create(context.pool(), &form).await?;
        Ok(comment.into())
    }
  }