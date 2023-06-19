use crate::{
    comment::CommentResponse,
    board::BoardResponse,
    data::TinyBoardsContext,
    post::PostResponse,
    utils::{/*check_person_block,*/ is_mod_or_admin},
};
use actix_web::web::Data;
use tinyboards_db::{
    models::{
        apub::actor_language::BoardLanguage,
        comment::comments::Comment,
        comment::comment_reply::{CommentReply, CommentReplyForm},
        person::person::Person,
        person::person_mentions::{PersonMention, PersonMentionForm},
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_db_views::structs::{CommentView, LocalUserView, PostView, BoardView};
use tinyboards_utils::{error::TinyBoardsError, utils::MentionData};

pub async fn build_comment_response(
    context: &Data<TinyBoardsContext>,
    comment_id: i32,
    local_user_view: Option<LocalUserView>,
    form_id: Option<String>,
    recipient_ids: Vec<i32>,
) -> Result<CommentResponse, TinyBoardsError> {
    let person_id = local_user_view.map(|l| l.person.id);
    let comment_view =  CommentView::read(context.pool(), comment_id, person_id).await?;
    Ok(CommentResponse { comment_view, recipient_ids, form_id })
}

pub async fn build_board_response(
    context: &Data<TinyBoardsContext>,
    local_user_view: LocalUserView,
    board_id: i32,
  ) -> Result<BoardResponse, TinyBoardsError> {
    let is_mod_or_admin = is_mod_or_admin(context.pool(), local_user_view.person.id, board_id)
      .await
      .is_ok();
    let person_id = local_user_view.person.id;
    let board_view = BoardView::read(
      context.pool(),
      board_id,
      Some(person_id),
      Some(is_mod_or_admin),
    )
    .await?;
    let discussion_languages = BoardLanguage::read(context.pool(), board_id).await?;
  
    Ok(BoardResponse {
      board_view,
      discussion_languages,
    })
}

pub async fn build_post_response(
    context: &Data<TinyBoardsContext>,
    board_id: i32,
    person_id: i32,
    post_id: i32,
  ) -> Result<PostResponse, TinyBoardsError> {
    let is_mod_or_admin = is_mod_or_admin(context.pool(), person_id, board_id)
      .await
      .is_ok();
    let post_view = PostView::read(
      context.pool(),
      post_id,
      Some(person_id),
      Some(is_mod_or_admin),
    )
    .await?;
    Ok(PostResponse { post_view })
}

#[tracing::instrument(skip_all)]
pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: &Comment,
  person: &Person,
  _post: &Post,
  _do_send_email: bool,
  context: &TinyBoardsContext,
) -> Result<Vec<i32>, TinyBoardsError> {
  let mut recipient_ids = Vec::new();
  let _inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name))
    .collect::<Vec<&MentionData>>()
  {
    let mention_name = mention.name.clone();
    let user_view = LocalUserView::get_by_name(context.pool(), &mention_name).await;
    if let Ok(mention_user_view) = user_view {
      // TODO
      // At some point, make it so you can't tag the parent creator either
      // This can cause two notifications, one for reply and the ofther for mention
      recipient_ids.push(mention_user_view.local_user.id);

      let user_mention_form = PersonMentionForm {
        recipient_id: Some(mention_user_view.person.id),
        comment_id: Some(comment.id),
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      PersonMention::create(context.pool(), &user_mention_form)
        .await
        .ok();

      // // Send an email to those local users that have notifications on
      // if do_send_email {
      //   let lang = get_interface_language(&mention_user_view);
      //   send_email_to_user(
      //     &mention_user_view,
      //     &lang.notification_mentioned_by_subject(&person.name),
      //     &lang.notification_mentioned_by_body(&comment.body, &inbox_link, &person.name),
      //     context.settings(),
      //   )
      // }
    }
  }
  Ok(recipient_ids)
}