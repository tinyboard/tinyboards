use porpl_db::{
    //aggregates::structs::{BoardAggregates, CommentAggregates, UserAggregates},
    models::{
        board::board::BoardSafe, /*comment::comment::Comment, comment::comment_reply::CommentReply,
        post::post::Post,*/ user::user::UserSafe, /*user::user_mention::UserMention*/,
    },
    //SubscribedType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardBlockView {
    pub user: UserSafe,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardSubscriberView {
    pub board: BoardSafe,
    pub subscriber: UserSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardModeratorView {
    pub board: BoardSafe,
    pub moderator: UserSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardUserBanView {
    pub board: BoardSafe,
    pub user: UserSafe,
}

/*#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardView {
    pub board: BoardSafe,
    pub subscribed: bool,
    pub blocked: bool,
    pub counts: BoardAggregates,
}*/

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBlockView {
    pub person: UserSafe,
    pub target: UserSafe,
}

/*#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserMentionView {
    pub user_mention: UserMention,
    pub comment: Comment,
    pub creator: UserSafe,
    pub post: Post,
    pub board: BoardSafe,
    pub recipient: UserSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool, // Left join BoardUserBan
    pub subscribed: SubscribedType,      // Left join to BoardSubscriber
    pub saved: bool,                     // Left join to CommentSaved
    pub creator_blocked: bool,           // Left join to UserBlock
    pub my_vote: Option<i16>,            // Left join to CommentLike
}*/

/*#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommentReplyView {
    pub comment_reply: CommentReply,
    pub comment: Comment,
    pub creator: UserSafe,
    pub post: Post,
    pub board: BoardSafe,
    pub recipient: UserSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool, // Left Join to BoardUserBan
    pub subscribed: SubscribedType,      // Left join to BoardSubscribers
    pub saved: bool,                     // Left join to CommentSaved
    pub creator_blocked: bool,           // Left join to PersonBlock
    pub my_vote: Option<i16>,            // Left join to CommentLike
}*/

/*#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserViewSafe {
    pub user: UserSafe,
    pub counts: UserAggregates,
}*/
