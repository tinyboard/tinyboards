use crate::{
    activities::board::send_activity_in_board,
    activity_lists::AnnouncableActivities,
    fetcher::post_or_comment::PostOrComment,
    objects::{comment::ApubComment, person::ApubPerson, post::ApubPost},
    protocol::activities::voting::{
      undo_vote::UndoVote,
      vote::{Vote, VoteType},
    },
    SendActivity,
  };
use tinyboards_db_views::structs::LocalUserView;
use tinyboards_federation::{config::Data, fetch::object_id::ObjectId};
use tinyboards_api_common::{
    comment::{CommentResponse, CreateCommentVote},
    data::TinyBoardsContext,
    post::{PostResponse, CreatePostVote},
    utils::require_user,
};
use tinyboards_db::{
    models::{
        comment::comment_votes::{CommentVote, CommentVoteForm},
        board::boards::Board,
        person::person::Person,
        post::post_votes::{PostVote, PostVoteForm},
    },
    traits::{Crud, Voteable},
};
use tinyboards_utils::error::TinyBoardsError;

pub mod undo_vote;
pub mod vote;

#[async_trait::async_trait]
impl SendActivity for CreatePostVote {
  type Response = PostResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let object_id = ObjectId::from(response.post_view.post.ap_id.unwrap().clone());
    let board_id = response.post_view.board.id;
    let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    send_activity(
      object_id, 
      board_id, 
      request.score, 
      view, 
      context
    )
    .await
  }
}

#[async_trait::async_trait]
impl SendActivity for CreateCommentVote {
  type Response = CommentResponse;

  async fn send_activity(
    request: &Self,
    response: &Self::Response,
    context: &Data<TinyBoardsContext>,
    auth: Option<&str>,
  ) -> Result<(), TinyBoardsError> {
    let object_id = ObjectId::from(response.comment_view.comment.ap_id.unwrap().clone());
    let board_id = response.comment_view.board.id;
    let view = require_user(context.pool(), context.master_key(), auth).await.unwrap()?;
    send_activity(
      object_id,
      board_id,
      request.score,
      view,
      context,
    )
    .await
  }
}

async fn send_activity(
    object_id: ObjectId<PostOrComment>,
    board_id: i32,
    score: i16,
    view: LocalUserView,
    context: &Data<TinyBoardsContext>,
  ) -> Result<(), TinyBoardsError> {
    let board = Board::read(context.pool(), board_id).await?.into();
    let actor = Person::read(context.pool(), view.person.id)
      .await?
      .into();
  
    // score of 1 means upvote, -1 downvote, 0 undo a previous vote
    if score != 0 {
      let vote = Vote::new(object_id, &actor, &board, score.try_into()?, context)?;
      let activity = AnnouncableActivities::Vote(vote);
      send_activity_in_board(activity, &actor, &board, vec![], false, context).await
    } else {
      // Tinyboards API doesnt distinguish between Undo/Like and Undo/Dislike, so we hardcode it here.
      let vote = Vote::new(object_id, &actor, &board, VoteType::Like, context)?;
      let undo_vote = UndoVote::new(vote, &actor, &board, context)?;
      let activity = AnnouncableActivities::UndoVote(undo_vote);
      send_activity_in_board(activity, &actor, &board, vec![], false, context).await
    }
  }

#[tracing::instrument(skip_all)]
async fn vote_comment(
  vote_type: &VoteType,
  actor: ApubPerson,
  comment: &ApubComment,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  let comment_id = comment.id;
  let vote_form = CommentVoteForm {
    comment_id,
    post_id: comment.post_id,
    person_id: actor.id,
    score: vote_type.into(),
  };
  let person_id = actor.id;
  CommentVote::remove(context.pool(), person_id, comment_id).await?;
  CommentVote::vote(context.pool(), &vote_form).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn vote_post(
  vote_type: &VoteType,
  actor: ApubPerson,
  post: &ApubPost,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  let post_id = post.id;
  let vote_form = PostVoteForm {
    post_id: post.id,
    person_id: actor.id,
    score: vote_type.into(),
  };
  let person_id = actor.id;
  PostVote::remove(context.pool(), person_id, post_id).await?;
  PostVote::vote(context.pool(), &vote_form).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn undo_vote_comment(
  actor: ApubPerson,
  comment: &ApubComment,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  let comment_id = comment.id;
  let person_id = actor.id;
  CommentVote::remove(context.pool(), person_id, comment_id).await?;
  Ok(())
}

#[tracing::instrument(skip_all)]
async fn undo_vote_post(
  actor: ApubPerson,
  post: &ApubPost,
  context: &Data<TinyBoardsContext>,
) -> Result<(), TinyBoardsError> {
  let post_id = post.id;
  let person_id = actor.id;
  PostVote::remove(context.pool(), person_id, post_id).await?;
  Ok(())
}