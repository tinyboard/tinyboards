use async_graphql::*;
use tinyboards_db::models::user::user::User as DbUser;
use tinyboards_db::models::aggregates::UserAggregates as DbUserAggregates;

/// GraphQL User type - public profile view.
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct User {
    pub id: ID,
    pub name: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    #[graphql(name = "bioHTML")]
    pub bio_html: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub profile_background: Option<String>,
    pub avatar_frame: Option<String>,
    pub profile_music: Option<String>,
    pub profile_music_youtube: Option<String>,
    pub signature: Option<String>,
    pub is_banned: bool,
    pub is_admin: bool,
    pub admin_level: i32,
    pub is_bot_account: bool,
    pub created_at: String,
    pub last_seen_at: String,
    pub unban_date: Option<String>,
    // Aggregate counts
    pub post_count: i64,
    pub post_score: i64,
    pub comment_count: i64,
    pub comment_score: i64,
    // Private fields for complex resolvers
    #[graphql(skip)]
    pub _is_application_accepted: bool,
    #[graphql(skip)]
    pub _is_board_creation_approved: bool,
    #[graphql(skip)]
    pub _has_verified_email: bool,
}

#[ComplexObject]
impl User {
    /// Whether user has a verified email (only visible to self or admin)
    pub async fn has_verified_email(&self, ctx: &Context<'_>) -> Option<bool> {
        use crate::helpers::permissions;
        let viewer = permissions::optional_auth(ctx);
        match viewer {
            Some(v) if v.id.to_string() == self.id.as_str() || v.is_admin => {
                Some(self._has_verified_email)
            }
            _ => None,
        }
    }

    /// Whether application is accepted (only visible to self or admin)
    pub async fn is_application_accepted(&self, ctx: &Context<'_>) -> Option<bool> {
        use crate::helpers::permissions;
        let viewer = permissions::optional_auth(ctx);
        match viewer {
            Some(v) if v.id.to_string() == self.id.as_str() || v.is_admin => {
                Some(self._is_application_accepted)
            }
            _ => None,
        }
    }

    /// Whether board creation is approved (only visible to self or admin)
    pub async fn is_board_creation_approved(&self, ctx: &Context<'_>) -> Option<bool> {
        use crate::helpers::permissions;
        let viewer = permissions::optional_auth(ctx);
        match viewer {
            Some(v) if v.id.to_string() == self.id.as_str() || v.is_admin => {
                Some(self._is_board_creation_approved)
            }
            _ => None,
        }
    }
}

impl User {
    pub fn from_db(user: DbUser, agg: Option<DbUserAggregates>) -> Self {
        let (post_count, post_score, comment_count, comment_score) = match agg {
            Some(a) => (a.post_count, a.post_score, a.comment_count, a.comment_score),
            None => (0, 0, 0, 0),
        };
        Self {
            id: ID(user.id.to_string()),
            name: user.name,
            display_name: user.display_name,
            bio: user.bio,
            bio_html: user.bio_html,
            avatar: user.avatar,
            banner: user.banner,
            profile_background: user.profile_background,
            avatar_frame: user.avatar_frame,
            profile_music: user.profile_music,
            profile_music_youtube: user.profile_music_youtube,
            signature: user.signature,
            is_banned: user.is_banned,
            is_admin: user.is_admin,
            admin_level: user.admin_level,
            is_bot_account: user.is_bot_account,
            created_at: user.created_at.to_rfc3339(),
            last_seen_at: user.last_seen_at.to_rfc3339(),
            unban_date: user.unban_date.map(|d| d.to_rfc3339()),
            post_count,
            post_score,
            comment_count,
            comment_score,
            _is_application_accepted: user.is_application_accepted,
            _is_board_creation_approved: user.is_board_creation_approved,
            _has_verified_email: user.is_email_verified,
        }
    }
}

/// User settings - private, only for the authenticated user.
#[derive(SimpleObject, Clone)]
pub struct UserSettings {
    pub id: ID,
    pub name: String,
    pub email: Option<String>,
    #[graphql(name = "showNSFW")]
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: String,
    pub default_listing_type: String,
    pub interface_language: String,
    pub is_email_notifications_enabled: bool,
    pub is_email_verified: bool,
    pub editor_mode: String,
    pub updated_at: String,
}

impl From<DbUser> for UserSettings {
    fn from(u: DbUser) -> Self {
        Self {
            id: ID(u.id.to_string()),
            name: u.name,
            email: u.email,
            show_nsfw: u.show_nsfw,
            show_bots: u.show_bots,
            theme: u.theme,
            default_sort_type: format!("{:?}", u.default_sort_type),
            default_listing_type: format!("{:?}", u.default_listing_type),
            interface_language: u.interface_language,
            is_email_notifications_enabled: u.is_email_notifications_enabled,
            is_email_verified: u.is_email_verified,
            editor_mode: format!("{:?}", u.editor_mode),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}
