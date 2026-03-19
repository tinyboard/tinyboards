use crate::schema::board_moderators;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Moderator permission levels, checked against the permissions bitmask column.
pub enum ModPerms {
    None,
    Config,
    Appearance,
    Content,
    Users,
    Emoji,
    Flair,
    Wiki,
    Full,
}

impl ModPerms {
    pub fn as_bitmask(&self) -> i32 {
        match self {
            ModPerms::None => 0,
            ModPerms::Config => 1 << 0,
            ModPerms::Appearance => 1 << 1,
            ModPerms::Content => 1 << 2,
            ModPerms::Users => 1 << 3,
            ModPerms::Emoji => 1 << 4,
            ModPerms::Flair => 1 << 5,
            ModPerms::Wiki => 1 << 6,
            ModPerms::Full => 0x7FFFFFFF,
        }
    }
}

/// A moderator assignment linking a user to a board with permissions.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = board_moderators)]
pub struct BoardModerator {
    pub id: Uuid,
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub permissions: i32,
    pub rank: i32,
    pub is_invite_accepted: bool,
    pub invite_accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl BoardModerator {
    /// Check if this moderator has the specified permission.
    pub fn has_permission(&self, perm: ModPerms) -> bool {
        let mask = perm.as_bitmask();
        if mask == 0 {
            return true;
        }
        // Full permission check: either has the specific bit or has Full
        (self.permissions & mask) != 0 || (self.permissions & ModPerms::Full.as_bitmask()) == ModPerms::Full.as_bitmask()
    }
}

/// Form for inserting a new board moderator.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = board_moderators)]
pub struct BoardModeratorInsertForm {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub permissions: i32,
    pub rank: i32,
    pub is_invite_accepted: bool,
    pub invite_accepted_at: Option<DateTime<Utc>>,
}

/// Form for updating an existing board moderator. Relationship FKs
/// (board_id, user_id) are not included since they should not change.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = board_moderators)]
pub struct BoardModeratorUpdateForm {
    pub permissions: Option<i32>,
    pub rank: Option<i32>,
    pub is_invite_accepted: Option<bool>,
    pub invite_accepted_at: Option<Option<DateTime<Utc>>>,
}
