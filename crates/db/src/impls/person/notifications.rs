use crate::schema::notifications::dsl::notifications;
use crate::{
    models::person::notifications::{Notification, NotificationForm, NotificationType},
    utils::{get_conn, DbPool},
};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

impl NotificationType {
    pub fn to_string(&self) -> String {
        use NotificationType::*;
        match self {
            PostReply(_) => "PostReply",
            CommentReply(_) => "CommentReply",
            UsernameMention(_) => "UsernameMention",
            Message(_) => "Message",
            NewPost(_) => "NewPost",
        }
        .to_owned()
    }
}

impl Notification {
    pub async fn send(pool: &DbPool, type_: NotificationType, send_to: i32) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;

        let notif_form = {
            use NotificationType::*;
            match type_ {
                PostReply(comment_id) | CommentReply(comment_id) | UsernameMention(comment_id) => {
                    NotificationForm {
                        kind: Some(type_.to_string()),
                        recipient_id: Some(send_to),
                        comment_id: Some(Some(comment_id)),
                        ..NotificationForm::default()
                    }
                }
                Message(message_id) => NotificationForm {
                    kind: Some(type_.to_string()),
                    recipient_id: Some(send_to),
                    message_id: Some(Some(message_id)),
                    ..NotificationForm::default()
                },
                NewPost(post_id) => NotificationForm {
                    kind: Some(type_.to_string()),
                    recipient_id: Some(send_to),
                    post_id: Some(Some(post_id)),
                    ..NotificationForm::default()
                },
            }
        };

        let new = diesel::insert_into(notifications)
            .values(notif_form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new).map(|_| ())
    }
}
