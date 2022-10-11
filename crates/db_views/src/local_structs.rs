use porpl_db::{
    aggregates::structs::{CommentAggregates, UserAggregates, PostAggregates},
    models::{
        comment::comment::Comment,
        board::board::BoardSafe,
        user::user::{User, UserSafe},
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
