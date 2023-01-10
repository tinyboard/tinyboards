pub mod board_moderator_view;
pub mod board_user_ban_view;
pub mod board_view;
pub mod comment_view;
pub mod post_view;
pub mod structs;
pub mod user_mention_view;
pub mod user_view;
pub mod site_invite_view;
pub mod site_view;
pub mod comment_reply_view;
pub mod private_message_view;

pub use comment_view::CommentQuery;

use tinyboards_db::models::user::users::User;
pub trait DeleteableOrRemoveable {
    fn hide_if_removed_or_deleted(&mut self, user_view: Option<&User>);
}
