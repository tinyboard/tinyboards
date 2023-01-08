use crate::{
    data::TinyBoardsContext,
    utils::blocking,  
};
use tinyboards_db::{
    models::{
        comment::comments::Comment,
        comment::comment_reply::{CommentReply, CommentReplyForm},
        user::{users::User, user_blocks::UserBlock},
        user::user_mentions::{UserMention, UserMentionForm},
        post::posts::Post,
    },
    traits::Crud,
};
use tinyboards_db_views::structs::{UserView};
use tinyboards_utils::{error::TinyBoardsError, utils::MentionData};


#[tracing::instrument(skip_all)]
pub async fn send_notifications(
    mentions: Vec<MentionData>,
    comment: &Comment,
    user: &User,
    post: &Post,
    context: &TinyBoardsContext,
) -> Result<Vec<i32>, TinyBoardsError> {
    let mut recipient_ids = Vec::new();
    
    // TODO - add in email logic eventually
    
    //let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

    for mention in mentions
        .iter()
        .filter(|m| m.name.ne(&user.name))
        .collect::<Vec<&MentionData>>() 
    {
        let mention_name = mention.name.clone();
        let user_view_res = blocking(context.pool(), move |conn| {
            UserView::read_from_name(conn, &mention_name)
        })
        .await?;

        match user_view_res {
            // we only proceed with trying to make a user mention if the user view returned successfully
            Ok(user_view) => {

                recipient_ids.push(user_view.user.id.clone());
                
                let user_mention_form = UserMentionForm {
                    recipient_id: Some(user_view.user.id.clone()),
                    comment_id: Some(comment.id),
                    read: Some(false)
                };

                // this might fail softly because comment edits might re-update or replace it
                // the table's uniqueness will handle the fail
                blocking(context.pool(), move |conn| {
                    UserMention::create(conn, &user_mention_form)
                })
                .await?
                .ok();

            },
            // do nothing if the user view lookup failed
            Err(_) => {
                ()
            }
        };        
    }

    // send comment reply to parent commenter/OP
    if let Some(parent_comment_id) = comment.parent_id {
        let parent_comment = blocking(context.pool(), move |conn| {
            Comment::read(conn, parent_comment_id)
        })
        .await??;

        let user_id = user.id.clone();
        let parent_creator_id = parent_comment.creator_id.clone();

        // only add to recipients if person is not blocked
        let creator_blocked = blocking(context.pool(), move |conn| {
            UserBlock::read(conn, user_id, parent_creator_id)
        })
        .await?
        .is_ok();

        // don't send a notification to yourself dummy
        if parent_comment.creator_id != user.id && !creator_blocked {
            let user_view = blocking(context.pool(), move |conn| {
                UserView::read(conn, parent_creator_id)
            })
            .await??;

            recipient_ids.push(user_view.user.id);

            let comment_reply_form = CommentReplyForm {
                recipient_id: Some(user_view.user.id),
                comment_id: Some(comment.id),
                read: Some(false),
            };

            // this needs to fail softly as well
            blocking(context.pool(), move |conn| {
                CommentReply::create(conn, &comment_reply_form)
            })
            .await?
            .ok();
            
        } 

    } else {
        // if no parent id then send a notification to the OP
        let user_id = user.id.clone();
        let post_creator_id = post.creator_id.clone();
        let creator_blocked = blocking(context.pool(), move |conn| {
            UserBlock::read(conn, user_id, post_creator_id)
        })
        .await?
        .is_ok();

        if post.creator_id != user.id && !creator_blocked {
            let creator_id = post.creator_id;
            let parent_user_view = blocking(context.pool(), move |conn| {
                UserView::read(conn, creator_id)
            })
            .await??;

            let comment_reply_form = CommentReplyForm {
                recipient_id: Some(parent_user_view.user.id),
                comment_id: Some(comment.id),
                read: Some(false),
            };

            // this needs to fail softly as well
            blocking(context.pool(), move |conn| {
                CommentReply::create(conn, &comment_reply_form)
            })
            .await?
            .ok();
        }

    }
    
    Ok(recipient_ids)
}