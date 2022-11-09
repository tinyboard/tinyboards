pub mod user;
pub mod user_block;



pub fn is_banned(banned_: bool, expires_: Option<chrono::NaiveDateTime>) -> bool {
    if let Some(expires_) = expires_ {
        banned_ && expires_.gt(&chrono::prelude::Utc::now().naive_utc())
    } else {
        banned_
    }
}