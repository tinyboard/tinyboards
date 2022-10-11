use crate::sensitive::Sensitive;
// use porpl_db::{
//     porpl_types::{CommentReplyId, BoardId, PersonId, PersonMentionId},
//     CommentSortType,
//     SortType,
// };
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Login {
    pub username_or_email: Sensitive<String>,
    pub password: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignupResponse {
    pub jwt: Option<Sensitive<String>>,
    pub registration_created: bool,
    pub verify_email_sent: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub jwt: Sensitive<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Register {
    pub username: String,
    pub password: Sensitive<String>,
    pub password_verify: Sensitive<String>,
    // pub show_nsfw: bool,
    // email = mandatory if email verification enabled on server
    pub email: Option<String>,
    pub captcha_uuid: Option<String>,
    pub captcha_answer: Option<String>,
    // An answer = required if require application is enabled on server
    pub answer: Option<String>,
}

#[derive(Deserialize)]
pub struct GetUser {}

#[derive(Deserialize)]
pub struct GetUserPath {
    pub username: String,
}
