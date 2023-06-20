use crate::{
    activities::{generate_activity_id, send_tinyboards_activity, verify_person_in_board},
    insert_activity,
    objects::{board::ApubBoard, person::ApubPerson},
    protocol::{activities::board::report::Report, InBoard},
    fetcher::post_or_comment::PostOrComment,
    SendActivity,
  };
  use tinyboards_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FlagType,
    traits::{ActivityHandler, Actor},
  };
  use tinyboards_api_common::{
    comment::{CommentReportResponse, CreateCommentReport},
    data::TinyBoardsContext,
    post::{CreatePostReport, PostReportResponse},
    utils::require_user,
  };
  use tinyboards_db::{
    models::{
      comment::comment_report::{CommentReport, CommentReportForm},
      post::post_report::{PostReport, PostReportForm},
    },
    traits::Reportable,
  };
  use tinyboards_utils::error::TinyBoardsError;
  use url::Url;

#[async_trait::async_trait]
impl SendActivity for CreatePostReport {
  type Response = PostReportResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    Report::send(
      ObjectId::from(response.post_report_view.post.ap_id.clone().unwrap()),
      &view.person.into(),
      ObjectId::from(response.post_report_view.board.actor_id.clone()),
      request.reason.to_string(),
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for CreateCommentReport {
  type Response = CommentReportResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    Report::send(
      ObjectId::from(response.comment_report_view.comment.ap_id.clone().unwrap()),
      &view.person.into(),
      ObjectId::from(response.comment_report_view.board.actor_id.clone()),
      request.reason.to_string(),
      context,
    )
    .await
  }
}

impl Report {
    #[tracing::instrument(skip_all)]
    async fn send(
      object_id: ObjectId<PostOrComment>,
      actor: &ApubPerson,
      board_id: ObjectId<ApubBoard>,
      reason: String,
      context: &Data<TinyBoardsContext>,
    ) -> Result<(), TinyBoardsError> {
      let board = board_id.dereference_local(context).await?;
      let kind = FlagType::Flag;
      let id = generate_activity_id(
        kind.clone(),
        &context.settings().get_protocol_and_hostname(),
      )?;
      let report = Report {
        actor: actor.id().into(),
        to: [board.id().into()],
        object: object_id,
        summary: reason,
        kind,
        id: id.clone(),
        audience: Some(board.id().into()),
      };
  
      let inbox = vec![board.shared_inbox_or_inbox()];
      send_tinyboards_activity(context, report, actor, inbox, false).await
    }
  }

#[async_trait::async_trait]
impl ActivityHandler for Report {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
    let board = self.board(context).await?;
    verify_person_in_board(&self.actor, &board, context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, true, context).await?;
    let actor = self.actor.dereference(context).await?;
    match self.object.dereference(context).await? {
      PostOrComment::Post(post) => {
        let report_form = PostReportForm {
          creator_id: Some(actor.id),
          post_id: Some(post.id),
          original_post_title: Some(post.title.clone()),
          original_post_url: Some(post.url.clone().unwrap()),
          reason: Some(self.summary),
          original_post_body: Some(post.body.clone()),
          ..PostReportForm::default()
        };
        PostReport::report(context.pool(), &report_form).await?;
      }
      PostOrComment::Comment(comment) => {
        let report_form = CommentReportForm {
          creator_id: Some(actor.id),
          comment_id: Some(comment.id),
          original_comment_text: Some(comment.body.clone()),
          reason: Some(self.summary),
          ..CommentReportForm::default()
        };
        CommentReport::report(context.pool(), &report_form).await?;
      }
    };
    Ok(())
  }
}