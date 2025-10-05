use crate::newtypes::DbUrl;
use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use typed_builder::TypedBuilder;

pub enum AdminPerms {
    Null,
    Appearance,
    Config,
    Content,
    Users,
    Boards,
    Emoji,
    Full,
    Owner,
    System,
}

#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub passhash: String,
    pub email_verified: bool,
    pub is_banned: bool,
    pub is_deleted: bool,
    pub is_admin: bool,
    pub admin_level: i32,
    pub unban_date: Option<NaiveDateTime>,
    pub bio: Option<String>,
    pub bio_html: Option<String>,
    pub signature: Option<String>,
    pub avatar: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub profile_background: Option<DbUrl>,
    pub avatar_frame: Option<DbUrl>,
    pub profile_music: Option<DbUrl>,
    pub profile_music_youtube: Option<String>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub interface_language: String,
    pub email_notifications_enabled: bool,
    pub bot_account: bool,
    pub board_creation_approved: bool,
    pub accepted_application: bool,
    pub is_application_accepted: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

/// A safe representation of user, without sensitive info like password hash and email
#[skip_serializing_none]
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct UserSafe {
    pub id: i32,
    pub name: String,
    pub display_name: Option<String>,
    pub is_banned: bool,
    pub is_deleted: bool,
    pub is_admin: bool,
    pub admin_level: i32,
    pub unban_date: Option<NaiveDateTime>,
    pub bio: Option<String>,
    pub bio_html: Option<String>,
    pub signature: Option<String>,
    pub avatar: Option<DbUrl>,
    pub banner: Option<DbUrl>,
    pub profile_background: Option<DbUrl>,
    pub avatar_frame: Option<DbUrl>,
    pub profile_music: Option<DbUrl>,
    pub profile_music_youtube: Option<String>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub interface_language: String,
    pub email_notifications_enabled: bool,
    pub bot_account: bool,
    pub board_creation_approved: bool,
    pub creation_date: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
}

/// Struct for retrieving setting columns from user table
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct UserSettings {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub email_notifications_enabled: bool,
    pub interface_language: String,
    pub updated: Option<NaiveDateTime>,
}

#[derive(
    Clone,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    Default,
    Insertable,
    AsChangeset,
    TypedBuilder,
)]
#[builder(field_defaults(default))]
#[diesel(table_name = users)]
pub struct UserForm {
    #[builder(!default)]
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<Option<String>>,
    #[builder(!default)]
    pub passhash: Option<String>,
    pub email_verified: Option<bool>,
    pub is_banned: Option<bool>,
    pub is_deleted: Option<bool>,
    pub is_admin: Option<bool>,
    pub admin_level: Option<i32>,
    pub unban_date: Option<Option<NaiveDateTime>>,
    pub bio: Option<String>,
    pub bio_html: Option<String>,
    pub signature: Option<Option<String>>,
    pub avatar: Option<Option<DbUrl>>,
    pub banner: Option<Option<DbUrl>>,
    pub profile_background: Option<Option<DbUrl>>,
    pub avatar_frame: Option<Option<DbUrl>>,
    pub profile_music: Option<Option<DbUrl>>,
    pub profile_music_youtube: Option<Option<String>>,
    pub bot_account: Option<bool>,
    pub board_creation_approved: Option<bool>,
    pub show_nsfw: Option<bool>,
    pub show_bots: Option<bool>,
    pub theme: Option<String>,
    pub default_sort_type: Option<i16>,
    pub default_listing_type: Option<i16>,
    pub interface_language: Option<String>,
    pub email_notifications_enabled: Option<bool>,
    pub accepted_application: Option<bool>,
    pub is_application_accepted: Option<bool>,
    pub creation_date: Option<NaiveDateTime>,
    pub updated: Option<Option<NaiveDateTime>>,
}

impl User {
    /// Check if user has the specified admin permission
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
            AdminPerms::Full => self.admin_level >= 6,
            AdminPerms::Owner => self.admin_level >= 7,
            AdminPerms::System => self.admin_level >= 8,
        }
    }

    /// Generate JWT token for this user - to be called from API layer with proper key
    /// This is a placeholder - the actual JWT generation should be done in the API layer
    pub fn get_jwt<T>(&self, _master_key: &T) -> String {
        // This should not be used directly - use the API utils auth::get_jwt function instead
        unimplemented!("Use tinyboards_api::utils::auth::get_jwt instead")
    }

    /// Update user's banned status
    pub async fn ban_user(
        pool: &crate::utils::DbPool,
        user_id: i32,
        is_banned: bool,
    ) -> Result<Self, tinyboards_utils::TinyBoardsError> {
        use crate::utils::get_conn;
        use crate::schema::users;
        let conn = &mut get_conn(pool).await?;

        use diesel_async::RunQueryDsl;

        let updated_user = diesel_async::RunQueryDsl::get_result(
            diesel::update(users::table.find(user_id))
                .set(users::is_banned.eq(is_banned)),
            conn
        )
        .await
            .map_err(|e| tinyboards_utils::TinyBoardsError::from(e))?;

        Ok(updated_user)
    }

}