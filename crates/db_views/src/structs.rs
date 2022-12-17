use tinyboards_db::{
    aggregates::structs::{BoardAggregates, CommentAggregates, PostAggregates, UserAggregates},
    models::{
        board::board::BoardSafe,
        comment::comment::Comment,
        post::post::Post,
        user::{user::{UserSafe, UserSettings}, user_mention::UserMention}, site::site_invite::SiteInvite,
    },
    SubscribedType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserView {
    pub user: UserSafe,
    pub counts: UserAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettingsView {
    pub settings: UserSettings,
    pub counts: UserAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PostView {
    pub post: Post,
    pub creator: Option<UserSafe>,
    pub board: BoardSafe,
    pub creator_banned_from_board: bool, // Left Join BoardUserBan
    pub counts: PostAggregates,
    pub subscribed: SubscribedType, // Left Join BoardSubscriber
    pub saved: bool,                // Left join PostSaved
    pub read: bool,                 // Left join PostRead
    pub creator_blocked: bool,      // Left join UserBlock
    pub my_vote: Option<i16>,       // Left join PostLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentView {
    pub comment: Comment,
    pub creator: Option<UserSafe>,
    pub post: Post,
    pub board: BoardSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool,
    pub subscribed: SubscribedType,
    pub saved: bool,
    pub creator_blocked: bool,
    pub my_vote: Option<i16>,
    pub replies: Vec<CommentView>,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardView {
    pub board: BoardSafe,
    pub subscribed: SubscribedType,
    pub blocked: bool,
    pub counts: BoardAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBlockView {
    pub person: UserSafe,
    pub target: UserSafe,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteInviteView {
    pub invite: SiteInvite,
}

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
