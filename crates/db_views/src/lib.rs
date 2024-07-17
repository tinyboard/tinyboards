pub mod board_moderator_view;
pub mod board_person_ban_view;
pub mod board_subscriber_view;
pub mod board_view;
pub mod comment_mod_queue_view;
pub mod comment_reply_view;
pub mod comment_report_view;
pub mod comment_view;
pub mod emoji_view;
pub mod local_user_view;
pub mod message_view;
pub mod person_mention_view;
pub mod person_view;
pub mod post_mod_queue_view;
pub mod post_report_view;
pub mod post_view;
pub mod registration_application_view;
pub mod site_invite_view;
pub mod site_view;
pub mod structs;

pub use comment_view::CommentQuery;

use structs::LocalUserView;
pub trait DeleteableOrRemoveable {
    fn hide_if_removed_or_deleted(&mut self, user_id: Option<i32>, is_admin: bool, is_mod: bool);
}
