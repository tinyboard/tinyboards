use crate::newtypes::{CommentId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
    feature = "full",
    diesel(belongs_to(crate::models::comment::comment::Comment))
)]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
/// This table keeps a list of replies to comments and posts.
pub struct CommentReply {
    pub id: CommentId,
    pub recipient_id: UserId,
    pub comment_id: CommentId,
    pub read: bool,
    pub creation_date: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
pub struct CommentReplyForm {
    pub recipient_id: UserId,
    pub comment_id: CommentId,
    pub read: Option<bool>,
}
