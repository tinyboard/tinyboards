use crate::enums::*;
use crate::schema::users;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A registered user account.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub passhash: String,
    pub is_email_verified: bool,
    pub is_banned: bool,
    pub is_admin: bool,
    pub admin_level: i32,
    pub is_bot_account: bool,
    pub is_board_creation_approved: bool,
    pub is_application_accepted: bool,
    pub unban_date: Option<DateTime<Utc>>,
    pub bio: Option<String>,
    pub bio_html: Option<String>,
    pub signature: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub profile_background: Option<String>,
    pub avatar_frame: Option<String>,
    pub profile_music: Option<String>,
    pub profile_music_youtube: Option<String>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: DbSortType,
    pub default_listing_type: DbListingType,
    pub interface_language: String,
    pub is_email_notifications_enabled: bool,
    pub editor_mode: DbEditorMode,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Admin permission levels, checked against the admin_level column.
pub enum AdminPerms {
    Null,
    Appearance,
    Config,
    Content,
    Users,
    Boards,
    Emoji,
    Flair,
    Full,
    Owner,
    System,
}

impl User {
    /// Check if user has the specified admin permission.
    pub fn has_permission(&self, perm: AdminPerms) -> bool {
        if !self.is_admin {
            return false;
        }

        match perm {
            AdminPerms::Null => true,
            AdminPerms::Appearance => self.admin_level >= 1,
            AdminPerms::Config => self.admin_level >= 2,
            AdminPerms::Content => self.admin_level >= 3,
            AdminPerms::Users => self.admin_level >= 4,
            AdminPerms::Boards => self.admin_level >= 5,
            AdminPerms::Emoji => self.admin_level >= 2,
            AdminPerms::Flair => self.admin_level >= 2,
            AdminPerms::Full => self.admin_level >= 6,
            AdminPerms::Owner => self.admin_level >= 7,
            AdminPerms::System => self.admin_level >= 8,
        }
    }
}

/// Form for inserting a new user. Required fields are not wrapped in Option.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct UserInsertForm {
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub passhash: String,
    pub is_email_verified: bool,
    pub is_banned: bool,
    pub is_admin: bool,
    pub admin_level: i32,
    pub is_bot_account: bool,
    pub is_board_creation_approved: bool,
    pub is_application_accepted: bool,
    pub unban_date: Option<DateTime<Utc>>,
    pub bio: Option<String>,
    pub bio_html: Option<String>,
    pub signature: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub profile_background: Option<String>,
    pub avatar_frame: Option<String>,
    pub profile_music: Option<String>,
    pub profile_music_youtube: Option<String>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: DbSortType,
    pub default_listing_type: DbListingType,
    pub interface_language: String,
    pub is_email_notifications_enabled: bool,
    pub editor_mode: DbEditorMode,
}

/// Form for updating an existing user. All fields are optional so only
/// changed columns are included in the UPDATE.
#[derive(Debug, Clone, Default, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserUpdateForm {
    pub name: Option<String>,
    pub display_name: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub passhash: Option<String>,
    pub is_email_verified: Option<bool>,
    pub is_banned: Option<bool>,
    pub is_admin: Option<bool>,
    pub admin_level: Option<i32>,
    pub is_bot_account: Option<bool>,
    pub is_board_creation_approved: Option<bool>,
    pub is_application_accepted: Option<bool>,
    pub unban_date: Option<Option<DateTime<Utc>>>,
    pub bio: Option<Option<String>>,
    pub bio_html: Option<Option<String>>,
    pub signature: Option<Option<String>>,
    pub avatar: Option<Option<String>>,
    pub banner: Option<Option<String>>,
    pub profile_background: Option<Option<String>>,
    pub avatar_frame: Option<Option<String>>,
    pub profile_music: Option<Option<String>>,
    pub profile_music_youtube: Option<Option<String>>,
    pub show_nsfw: Option<bool>,
    pub show_bots: Option<bool>,
    pub theme: Option<String>,
    pub default_sort_type: Option<DbSortType>,
    pub default_listing_type: Option<DbListingType>,
    pub interface_language: Option<String>,
    pub is_email_notifications_enabled: Option<bool>,
    pub editor_mode: Option<DbEditorMode>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<Option<DateTime<Utc>>>,
}
