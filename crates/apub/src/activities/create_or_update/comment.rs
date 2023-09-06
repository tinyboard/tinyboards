use crate::{
    activities::{
      check_board_deleted_or_removed,
      board::send_activity_in_board,
      generate_activity_id,
      verify_is_public,
      verify_person_in_board,
    },
    activity_lists::AnnouncableActivities,
    insert_activity,
    mentions::MentionOrValue,
    objects::{comment::ApubComment, board::ApubBoard, person::ApubPerson},
    protocol::{
      activities::{create_or_update::note::CreateOrUpdateNote, CreateOrUpdateType},
      InBoard,
    },
    SendActivity,
};
use tinyboards_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::public,
  protocol::verification::verify_domains_match,
  traits::{ActivityHandler, Actor, Object},
};
use tinyboards_api_common::{
  build_response::send_local_notifs,
  comment::{CommentResponse, CreateComment, EditComment},
  data::TinyBoardsContext,
  utils::check_post_deleted_or_removed,
};
use tinyboards_db::{
  models::{
    comment::{
      comments::Comment,
      comment_votes::{CommentVote, CommentVoteForm},
    },
    board::boards::Board,
    person::person::Person,
    post::posts::Post,
  },
  traits::{Crud, Voteable},
};
use tinyboards_utils::{error::TinyBoardsError, utils::scrape_text_for_mentions};
use url::Url;

use tracing::info;

#[async_trait::async_trait]
impl SendActivity for CreateComment {
  type Response = CommentResponse;

  async fn send_activity(
    _request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    _auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    CreateOrUpdateNote::send(
      &response.comment_view.comment,
      response.comment_view.creator.clone().unwrap().id,
      CreateOrUpdateType::Create,
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for EditComment {
  type Response = CommentResponse;

  async fn send_activity(
    _request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    _auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    CreateOrUpdateNote::send(
      &response.comment_view.comment,
      response.comment_view.creator.clone().unwrap().id,
      CreateOrUpdateType::Update,
      context,
    )
    .await
  }
}

impl CreateOrUpdateNote {
  #[tracing::instrument(skip(comment, person_id, kind, context))]
  async fn send(
    comment: &Comment,
    person_id: i32,
    kind: CreateOrUpdateType,
    context: &Data<TinyBoardsContext>,
  ) -> Result<(), TinyBoardsError> {
    // TODO: might be helpful to add a comment method to retrieve board directly
    let post_id = comment.post_id;
    let post = Post::read(context.pool(), post_id).await?;
    let board_id = post.board_id;
    let person: ApubPerson = Person::read(context.pool(), person_id).await?.into();
    let board: ApubBoard = Board::read(context.pool(), board_id).await?.into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let note = ApubComment(comment.clone()).into_json(context).await?;

    let create_or_update = CreateOrUpdateNote {
      actor: person.id().into(),
      to: vec![public()],
      cc: note.cc.clone(),
      tag: note.tag.clone(),
      object: note,
      kind,
      id: id.clone(),
      audience: Some(board.id().into()),
    };

    let tagged_users: Vec<ObjectId<ApubPerson>> = create_or_update
      .tag
      .iter()
      .filter_map(|t| {
        if let MentionOrValue::Mention(t) = t {
          Some(t)
        } else {
          None
        }
      })
      .map(|t| t.href.clone())
      .map(ObjectId::from)
      .collect();
    let mut inboxes = vec![];
    for t in tagged_users {
      let person = t.dereference(context).await?;
      inboxes.push(person.shared_inbox_or_inbox());
    }

    let activity = AnnouncableActivities::CreateOrUpdateComment(create_or_update);
    send_activity_in_board(activity, &person, &board, inboxes, false, context).await
  }
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateNote {
  type DataType = TinyBoardsContext;
  type Error = TinyBoardsError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    verify_is_public(&self.to, &self.cc)?;
    let post = self.object.get_parents(context).await?.0;
    let board = self.board(context).await?;

    verify_person_in_board(&self.actor, &board, context).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    check_board_deleted_or_removed(&board)?;
    check_post_deleted_or_removed(post.id, context.pool()).await?;

    ApubComment::verify(&self.object, self.actor.inner(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<TinyBoardsContext>) -> Result<(), TinyBoardsError> {
    insert_activity(&self.id, &self, false, false, context).await?;
    let comment = ApubComment::from_json(self.object, context).await?;

    // author upvotes their own comment by default
    let vote_form = CommentVoteForm {
      comment_id: comment.id,
      post_id: comment.post_id,
      person_id: comment.creator_id,
      score: 1,
    };
    CommentVote::vote(context.pool(), &vote_form).await?;

    let do_send_email = self.kind == CreateOrUpdateType::Create;
    let post_id = comment.post_id;
    let post = Post::read(context.pool(), post_id).await?;
    let actor = self.actor.dereference(context).await?;

    // Note:
    // Although mentions could be gotten from the post tags (they are included there), or the ccs,
    // Its much easier to scrape them from the comment body, since the API has to do that
    // anyway.
    // TODO: for compatibility with other projects, it would be much better to read this from cc or tags
    let mentions = scrape_text_for_mentions(&comment.body);
    send_local_notifs(mentions, &comment.0, &actor, &post, do_send_email, context).await?;
    Ok(())
  }
}