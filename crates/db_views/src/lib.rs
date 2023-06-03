pub mod board_moderator_view;
pub mod board_person_ban_view;
pub mod board_view;
pub mod comment_view;
pub mod post_view;
pub mod structs;
pub mod person_mention_view;
pub mod person_view;
pub mod site_invite_view;
pub mod site_view;
pub mod comment_reply_view;
pub mod registration_application_view;

pub use comment_view::CommentQuery;

use tinyboards_db::models::person::local_user::LocalUser;
pub trait DeleteableOrRemoveable {
    fn hide_if_removed_or_deleted(&mut self, user_view: Option<&LocalUser>);
}
