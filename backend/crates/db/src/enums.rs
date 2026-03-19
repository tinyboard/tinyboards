// Diesel-compatible enum types that map to PostgreSQL ENUM types.
// Each enum implements ToSql and FromSql for the corresponding sql_types type
// defined in schema.rs.

use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::schema::sql_types;

// ============================================================
// Macro to reduce boilerplate for simple string-backed enums
// ============================================================

macro_rules! pg_enum {
    (
        $sql_type:ty,
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $pg_value:expr ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $( $(#[$vmeta])* $variant ),+
        }

        impl ToSql<$sql_type, Pg> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
                match self {
                    $( Self::$variant => out.write_all($pg_value)?, )+
                }
                Ok(IsNull::No)
            }
        }

        impl FromSql<$sql_type, Pg> for $name {
            fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
                match bytes.as_bytes() {
                    $( $pg_value => Ok(Self::$variant), )+
                    other => Err(format!(
                        "Unrecognized {} variant: {:?}",
                        stringify!($name),
                        String::from_utf8_lossy(other)
                    ).into()),
                }
            }
        }
    };
}

pg_enum! {
    sql_types::RegistrationMode,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::RegistrationMode)]
    pub enum DbRegistrationMode {
        Open => b"open",
        InviteOnly => b"invite_only",
        ApplicationRequired => b"application_required",
        Closed => b"closed",
    }
}

pg_enum! {
    sql_types::PostType,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::PostType)]
    pub enum DbPostType {
        Text => b"text",
        Link => b"link",
        Image => b"image",
        Video => b"video",
    }
}

pg_enum! {
    sql_types::SortType,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::SortType)]
    pub enum DbSortType {
        Hot => b"hot",
        New => b"new",
        Top => b"top",
        Old => b"old",
        MostComments => b"most_comments",
        Controversial => b"controversial",
    }
}

pg_enum! {
    sql_types::ListingType,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::ListingType)]
    pub enum DbListingType {
        All => b"all",
        Subscribed => b"subscribed",
        Local => b"local",
    }
}

pg_enum! {
    sql_types::ApprovalStatus,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::ApprovalStatus)]
    pub enum DbApprovalStatus {
        Pending => b"pending",
        Approved => b"approved",
        Rejected => b"rejected",
    }
}

pg_enum! {
    sql_types::EditorMode,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::EditorMode)]
    pub enum DbEditorMode {
        RichText => b"richtext",
        Markdown => b"markdown",
        PlainText => b"plaintext",
    }
}

pg_enum! {
    sql_types::NotificationKind,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::NotificationKind)]
    pub enum DbNotificationKind {
        CommentReply => b"comment_reply",
        PostReply => b"post_reply",
        Mention => b"mention",
        PrivateMessage => b"private_message",
        ModAction => b"mod_action",
        System => b"system",
    }
}

pg_enum! {
    sql_types::ModerationAction,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::ModerationAction)]
    pub enum DbModerationAction {
        BanUser => b"ban_user",
        UnbanUser => b"unban_user",
        BanFromBoard => b"ban_from_board",
        UnbanFromBoard => b"unban_from_board",
        RemovePost => b"remove_post",
        RestorePost => b"restore_post",
        RemoveComment => b"remove_comment",
        RestoreComment => b"restore_comment",
        LockPost => b"lock_post",
        UnlockPost => b"unlock_post",
        LockComment => b"lock_comment",
        UnlockComment => b"unlock_comment",
        FeaturePost => b"feature_post",
        UnfeaturePost => b"unfeature_post",
        RemoveBoard => b"remove_board",
        RestoreBoard => b"restore_board",
        HideBoard => b"hide_board",
        UnhideBoard => b"unhide_board",
        AddMod => b"add_mod",
        RemoveMod => b"remove_mod",
        AddAdmin => b"add_admin",
        RemoveAdmin => b"remove_admin",
        PurgeUser => b"purge_user",
        PurgePost => b"purge_post",
        PurgeComment => b"purge_comment",
        PurgeBoard => b"purge_board",
    }
}

pg_enum! {
    sql_types::WikiPermission,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::WikiPermission)]
    pub enum DbWikiPermission {
        Public => b"public",
        Members => b"members",
        ModsOnly => b"mods_only",
        Locked => b"locked",
    }
}

pg_enum! {
    sql_types::FlairType,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::FlairType)]
    pub enum DbFlairType {
        Post => b"post",
        User => b"user",
    }
}

pg_enum! {
    sql_types::FilterMode,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::FilterMode)]
    pub enum DbFilterMode {
        Include => b"include",
        Exclude => b"exclude",
    }
}

pg_enum! {
    sql_types::EmojiScope,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::EmojiScope)]
    pub enum DbEmojiScope {
        Global => b"global",
        Board => b"board",
    }
}

pg_enum! {
    sql_types::ReportStatus,
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::ReportStatus)]
    pub enum DbReportStatus {
        Pending => b"pending",
        Resolved => b"resolved",
        Dismissed => b"dismissed",
    }
}
