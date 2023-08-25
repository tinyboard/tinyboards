use crate::{
    data::TinyBoardsContext,
};
use tinyboards_db::{
    models::{
        comment::comments::Comment,
        comment::comment_reply::{CommentReply, CommentReplyForm},
        person::{person::*, person_blocks::*, person_mentions::*,},
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_db_views::structs::PersonView;
use tinyboards_utils::{error::TinyBoardsError, utils::MentionData};


#[tracing::instrument(skip_all)]
pub async fn send_notifications(
    mentions: Vec<MentionData>,
    comment: &Comment,
    person: &Person,
    post: &Post,
    context: &TinyBoardsContext,
) -> Result<Vec<i32>, TinyBoardsError> {
    let mut recipient_ids = Vec::new();
    
    // TODO - add in email logic eventually
    
    //let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

    for mention in mentions
        .iter()
        .filter(|m| m.name.ne(&person.name))
        .collect::<Vec<&MentionData>>() 
    {
        let mention_name = mention.name.clone();

        let person_view_res = PersonView::read_from_name(context.pool(), &mention_name).await;

        match person_view_res {
            // we only proceed with trying to make a user mention if the user view returned successfully
            Ok(person_view) => {

                recipient_ids.push(person_view.person.id.clone());
                
                let user_mention_form = PersonMentionForm {
                    recipient_id: Some(person_view.person.id.clone()),
                    comment_id: Some(comment.id),
                    read: Some(false)
                };

                // this might fail softly because comment edits might re-update or replace it
                // the table's uniqueness will handle the fail
                PersonMention::create(context.pool(), &user_mention_form).await.ok();

            },
            // do nothing if the user view lookup failed
            Err(_) => {
                ()
            }
        };        
    }

    // send comment reply to parent commenter/OP
    if let Some(parent_comment_id) = comment.parent_id {
        let parent_comment = Comment::read(context.pool(), parent_comment_id).await?;
        let person_id = person.id.clone();
        let parent_creator_id = parent_comment.creator_id.clone();

        // only add to recipients if person is not blocked
        let creator_blocked = PersonBlock::read(context.pool(), person_id, parent_creator_id).await.is_ok();

        // don't send a notification to yourself dummy
        if parent_comment.creator_id != person.id && !creator_blocked {

            let person_view = PersonView::read(context.pool(), parent_creator_id, false).await?;
            recipient_ids.push(person_view.person.id);

            let comment_reply_form = CommentReplyForm {
                recipient_id: Some(person_view.person.id),
                comment_id: Some(comment.id),
                read: Some(false),
            };

            // this needs to fail softly as well
            CommentReply::create(context.pool(), &comment_reply_form).await.ok();
            
        } 

    } else {
        // if no parent id then send a notification to the OP
        let person_id = person.id.clone();
        let post_creator_id = post.creator_id.clone();
        let creator_blocked = PersonBlock::read(context.pool(), person_id, post_creator_id).await.is_ok();

        if post.creator_id != person.id && !creator_blocked {
            let creator_id = post.creator_id;
            let parent_person_view = PersonView::read(context.pool(), creator_id, false).await?;

            let comment_reply_form = CommentReplyForm {
                recipient_id: Some(parent_person_view.person.id),
                comment_id: Some(comment.id),
                read: Some(false),
            };

            // this needs to fail softly as well
            CommentReply::create(context.pool(), &comment_reply_form).await.ok();
        }

    }
    
    Ok(recipient_ids)
}