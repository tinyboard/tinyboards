use porpl_db::{
    aggregates::structs::{CommentAggregates, UserAggregates, PostAggregates},
    models::{
        comment::comment::Comment,
        board::board::BoardSafe,
        user::user::{User, UserSafe, UserSettings},
        post::post::Post,
    },
    SubscribedType,
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserView {
    pub user: User,
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
    pub creator: UserSafe,
    pub board: BoardSafe,
    pub creator_banned_from_board: bool, // Left Join BoardUserBan
    pub counts: PostAggregates,
    pub subscribed: SubscribedType, // Left Join BoardSubscriber
    pub saved: bool, // Left join PostSaved
    pub read: bool, // Left join PostRead
    pub creator_blocked: bool, // Left join UserBlock
    pub my_vote: Option<i16>, // Left join PostLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentView {
    pub comment: Comment,
    pub creator: UserSafe,
    pub post: Post,
    pub board: BoardSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool, 
    pub subscribed: SubscribedType,
    pub saved: bool,
    pub creator_blocked: bool,
    pub my_vote: Option<i16>,
}