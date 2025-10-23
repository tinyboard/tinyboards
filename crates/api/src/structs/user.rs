use async_graphql::*;
use tinyboards_db::{
    aggregates::structs::UserAggregates as DbUserAggregates,
    models::{
        board::{board_mods::BoardModerator as DbBoardMod, boards::Board as DbBoard},
        comment::comments::Comment as DbComment,
        user::user::{User as DbUser, AdminPerms},
        post::posts::Post as DbPost,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{CommentSortType, ListingType, LoggedInUser, SortType};

use super::{board_mods::BoardMod, boards::Board, comment::Comment, post::Post};

/// User settings
#[derive(SimpleObject, Clone)]
struct Settings {
    email: Option<String>,
    email_notifications_enabled: bool,
    default_sort_type: i16,
    default_listing_type: i16,
    #[graphql(name = "showNSFW")]
    show_nsfw: bool,
    show_bots: bool,
}

/// Public UserSettings object (for GraphQL queries)
#[derive(SimpleObject, Clone)]
pub struct UserSettings {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    #[graphql(name = "showNSFW")]
    pub show_nsfw: bool,
    pub show_bots: bool,
    pub theme: String,
    pub default_sort_type: i16,
    pub default_listing_type: i16,
    pub email_notifications_enabled: bool,
    pub interface_language: String,
    pub updated: Option<String>,
}

impl From<tinyboards_db::models::user::user::UserSettings> for UserSettings {
    fn from(settings: tinyboards_db::models::user::user::UserSettings) -> Self {
        Self {
            id: settings.id,
            name: settings.name,
            email: settings.email,
            show_nsfw: settings.show_nsfw,
            show_bots: settings.show_bots,
            theme: settings.theme,
            default_sort_type: settings.default_sort_type,
            default_listing_type: settings.default_listing_type,
            email_notifications_enabled: settings.email_notifications_enabled,
            interface_language: settings.interface_language,
            updated: settings.updated.map(|u| u.to_string()),
        }
    }
}

/// GraphQL representation of User.
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct User {
    pub id: i32,
    name: String,
    is_banned: bool,
    is_active: bool,
    unban_date: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    #[graphql(name = "bioHTML")]
    bio_html: Option<String>,
    creation_date: String,
    updated: Option<String>,
    last_seen: String,
    avatar: Option<String>,
    banner: Option<String>,
    profile_background: Option<String>,
    #[graphql(skip)]
    pub is_admin: bool,
    admin_level: i32,
    is_local: bool,
    instance: Option<String>,
    profile_music: Option<String>,
    profile_music_youtube: Option<String>,
    signature: Option<String>,
    // Private fields for additional admin things
    #[graphql(skip)]
    _has_verified_email: Option<bool>,
    #[graphql(skip)]
    _is_application_accepted: bool,
    #[graphql(skip)]
    _board_creation_approved: bool,
    // Settings: only available for your own account
    #[graphql(skip)]
    _settings: Option<Settings>,
    // `counts` is not queryable, instead, its fields are available for User thru dynamic resolvers
    #[graphql(skip)]
    counts: DbUserAggregates,
}

/// Own profile
#[derive(SimpleObject)]
pub struct Me {
    pub user: Option<User>,
    pub unread_replies_count: Option<i64>,
    pub unread_mentions_count: Option<i64>,
}

// resolvers for UserAggregates fields
#[ComplexObject]
impl User {
    pub async fn post_count(&self) -> i64 {
        self.counts.post_count
    }

    pub async fn comment_count(&self) -> i64 {
        self.counts.comment_count
    }

    pub async fn post_score(&self) -> i64 {
        self.counts.post_score
    }

    pub async fn comment_score(&self) -> i64 {
        self.counts.comment_score
    }

    pub async fn rep(&self) -> i64 {
        // Calculate reputation from post and comment scores
        self.counts.post_score + self.counts.comment_score
    }

    pub async fn is_admin(&self) -> bool {
        self.admin_level > 0
    }

    pub async fn has_verified_email(&self, ctx: &Context<'_>) -> Option<bool> {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id == self.id || v.admin_level >= AdminPerms::Users as i32 {
            self._has_verified_email
        } else {
            None
        }
    }

    pub async fn is_application_accepted(&self, ctx: &Context<'_>) -> Option<bool> {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id == self.id || v.admin_level >= AdminPerms::Users as i32 {
            Some(self._is_application_accepted)
        } else {
            None
        }
    }

    /// Check if user has a pending application (visible to self and admins)
    pub async fn has_pending_application(&self, ctx: &Context<'_>) -> Option<bool> {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id == self.id || v.admin_level >= AdminPerms::Users as i32 {
            use tinyboards_db::models::site::registration_applications::RegistrationApplication;
            let pool = ctx.data_unchecked::<DbPool>();

            // Check if there's an application for this user
            let has_application = RegistrationApplication::find_by_user_id(pool, self.id)
                .await
                .is_ok();

            Some(has_application)
        } else {
            None
        }
    }

    /// Board creation approval status (admin-only or self)
    pub async fn board_creation_approved(&self, ctx: &Context<'_>) -> Option<bool> {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id == self.id || v.admin_level >= AdminPerms::Users as i32 {
            Some(self._board_creation_approved)
        } else {
            None
        }
    }

    /// Get flairs assigned to this user
    pub async fn flairs(&self, ctx: &Context<'_>) -> Result<Vec<super::flair::UserFlair>> {
        use tinyboards_db::{models::flair::user_flair::UserFlair as DbUserFlair, schema::user_flairs, utils::get_conn};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get database connection"))?;

        let user_flairs = user_flairs::table
            .filter(user_flairs::user_id.eq(self.id))
            .filter(user_flairs::is_approved.eq(true))
            .load::<DbUserFlair>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load user flairs"))?;

        Ok(user_flairs.into_iter().map(super::flair::UserFlair::from).collect())
    }

    pub async fn settings(&self, ctx: &Context<'_>) -> Option<&Settings> {
        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id == self.id {
            self._settings.as_ref()
        } else {
            None
        }
    }

    pub async fn joined_boards(
        &self,
        ctx: &Context<'_>,
        limit: Option<i64>,
        page: Option<i64>,
    ) -> Result<Vec<Board>> {
        let pool = ctx.data_unchecked::<DbPool>();
        let v = ctx.data_unchecked::<LoggedInUser>().require_user()?;
        let my_user_id = v.id;

        if self.id != my_user_id {
            return Err(TinyBoardsError::from_message(
                400,
                "You cannot see what boards someone else has joined, moron.",
            )
            .into());
        }

        DbBoard::list_with_counts(
            pool,
            my_user_id,
            limit,
            page,
            SortType::Active.into(),
            ListingType::Subscribed.into(),
            None,
            false,
            false,
        )
        .await
        .map(|res| res.into_iter().map(Board::from).collect::<Vec<Board>>())
        .map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Failed to load joined boards").into()
        })
    }

    /// Get unread replies count for this user (only visible to self)
    pub async fn unread_replies_count(&self, ctx: &Context<'_>) -> Option<i64> {
        use tinyboards_db::models::notification::notifications::Notification as DbNotification;

        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id != self.id {
            return None; // Only show your own unread counts
        }

        let pool = ctx.data_unchecked::<DbPool>();

        // Get counts for comment_reply and post_reply
        match DbNotification::count_unread_by_kind(pool, self.id).await {
            Ok(counts) => {
                let reply_count: i64 = counts
                    .iter()
                    .filter(|(kind, _)| kind == "comment_reply" || kind == "post_reply")
                    .map(|(_, count)| *count)
                    .sum();
                Some(reply_count)
            }
            Err(_) => Some(0),
        }
    }

    /// Get unread mentions count for this user (only visible to self)
    pub async fn unread_mentions_count(&self, ctx: &Context<'_>) -> Option<i64> {
        use tinyboards_db::models::notification::notifications::Notification as DbNotification;

        let v_opt = ctx.data_unchecked::<LoggedInUser>().inner();

        if v_opt.is_none() {
            return None;
        }

        let v = v_opt.unwrap();
        if v.id != self.id {
            return None; // Only show your own unread counts
        }

        let pool = ctx.data_unchecked::<DbPool>();

        // Get count for mention notifications
        match DbNotification::count_unread_by_kind(pool, self.id).await {
            Ok(counts) => {
                let mention_count: i64 = counts
                    .iter()
                    .filter(|(kind, _)| kind == "mention")
                    .map(|(_, count)| *count)
                    .sum();
                Some(mention_count)
            }
            Err(_) => Some(0),
        }
    }

    pub async fn moderates(&self, ctx: &Context<'_>) -> Result<Vec<BoardMod>> {
        let pool = ctx.data_unchecked::<DbPool>();

        DbBoardMod::for_user(pool, self.id)
            .await
            .map(|res| {
                res.into_iter()
                    .map(BoardMod::from)
                    .collect::<Vec<BoardMod>>()
            })
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to load modded boards.").into()
            })
    }

    pub async fn posts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many posts to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Sorting type.")] sort: Option<SortType>,
        #[graphql(desc = "If specified, only posts from the given user will be loaded.")]
        board_id: Option<i32>,
        #[graphql(desc = "Page.")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let sort = sort.unwrap_or(SortType::NewComments);
        let listing_type = ListingType::All;
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let user_id_join = match v_opt {
            Some(v) => v.id,
            None => -1,
        };

        let is_admin = match v_opt {
            Some(v) => v.admin_level >= AdminPerms::Boards as i32 || v.admin_level >= AdminPerms::Content as i32,
            None => false,
        };

        let is_self = match v_opt {
            Some(v) => self.id == v.id,
            None => false,
        };

        // Post history of banned users is hidden
        if self.is_banned && !(is_admin || is_self) {
            return Ok(vec![]);
        }

        let (include_deleted, include_removed, include_banned_boards) = if is_admin {
            // admins see everything
            (true, true, true)
        } else if is_self {
            // you can see your own removed content, but not posts in banned boards, and posts that you've deleted
            (false, true, false)
        } else {
            // logged out; or logged in, but neither self nor admin
            (false, false, false)
        };

        let posts = DbPost::load_with_counts(
            pool,
            user_id_join,
            Some(limit),
            page,
            include_deleted,
            include_removed,
            include_banned_boards,
            false,
            board_id,
            Some(self.id),
            sort.into(),
            listing_type.into(),
        )
        .await?;

        Ok(posts.into_iter().map(Post::from).collect::<Vec<Post>>())
    }

    pub async fn comments(
        &self,
        ctx: &Context<'_>,
        sort: Option<CommentSortType>,
        limit: Option<i64>,
        page: Option<i64>,
    ) -> Result<Vec<Comment>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let sort = sort.unwrap_or(CommentSortType::New);
        let listing_type = ListingType::All;
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let user_id_join = match v_opt {
            Some(v) => v.id,
            None => -1,
        };

        let is_admin = match v_opt {
            Some(v) => v.admin_level >= AdminPerms::Boards as i32 || v.admin_level >= AdminPerms::Content as i32,
            None => false,
        };

        let is_self = match v_opt {
            Some(v) => self.id == v.id,
            None => false,
        };

        // Comment history of banned users is hidden
        if self.is_banned && !(is_admin || is_self) {
            return Ok(vec![]);
        }

        let (include_deleted, include_removed, include_banned_boards) = if is_admin {
            // admins see everything
            (true, true, true)
        } else if is_self {
            // you can see your own removed content, but not comments in banned boards, and comments that you've deleted
            (false, true, false)
        } else {
            // logged out; or logged in, but neither self nor admin
            (false, false, false)
        };

        let comments = DbComment::load_with_counts(
            pool,
            user_id_join,
            sort.into(),
            listing_type.into(),
            page,
            Some(limit),
            Some(self.id),
            None,
            None,
            false,
            None,
            include_deleted,
            include_removed,
            include_banned_boards,
            None,
            None,
        )
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to load comments"))?;

        Ok(comments
            .into_iter()
            .map(Comment::from)
            .collect::<Vec<Comment>>())
    }
}

impl User {
    /// Check if user has the specified admin permission level
    pub fn has_permission(&self, perm: tinyboards_db::models::user::user::AdminPerms) -> bool {
        tracing::debug!(
            "User::has_permission - user_id={}, is_admin={}, admin_level={}",
            self.id, self.is_admin, self.admin_level
        );

        if !self.is_admin {
            tracing::debug!("User::has_permission - DENIED: is_admin is false");
            return false;
        }

        let result = match perm {
            tinyboards_db::models::user::user::AdminPerms::Null => true,
            tinyboards_db::models::user::user::AdminPerms::Appearance => self.admin_level >= 1,
            tinyboards_db::models::user::user::AdminPerms::Config => self.admin_level >= 2,
            tinyboards_db::models::user::user::AdminPerms::Content => self.admin_level >= 3,
            tinyboards_db::models::user::user::AdminPerms::Users => self.admin_level >= 4,
            tinyboards_db::models::user::user::AdminPerms::Boards => self.admin_level >= 5,
            tinyboards_db::models::user::user::AdminPerms::Emoji => self.admin_level >= 2,
            tinyboards_db::models::user::user::AdminPerms::Flair => self.admin_level >= 2,
            tinyboards_db::models::user::user::AdminPerms::Full => self.admin_level >= 6,
            tinyboards_db::models::user::user::AdminPerms::Owner => self.admin_level >= 7,
            tinyboards_db::models::user::user::AdminPerms::System => self.admin_level >= 8,
        };

        tracing::debug!("User::has_permission - result={}", result);
        result
    }
}

impl From<(DbUser, DbUserAggregates)> for User {
    fn from((user, counts): (DbUser, DbUserAggregates)) -> Self {
        Self {
            id: user.id,
            name: user.name,
            is_banned: user.is_banned,
            is_active: !user.is_deleted,
            unban_date: user.unban_date.map(|t| t.to_string()),
            display_name: user.display_name,
            bio: user.bio,
            bio_html: user.bio_html,
            creation_date: user.creation_date.to_string(),
            updated: user.updated.map(|t| t.to_string()),
            last_seen: user.last_seen.to_string(),
            avatar: user.avatar.map(|a| a.as_str().into()),
            banner: user.banner.map(|a| a.as_str().into()),
            profile_background: user.profile_background.map(|a| a.as_str().into()),
            is_admin: user.is_admin,
            admin_level: user.admin_level,
            is_local: true, // All users are local in consolidated model
            instance: None, // No instance field in consolidated User model
            profile_music: user.profile_music.map(|m| m.as_str().into()),
            profile_music_youtube: user.profile_music_youtube,
            signature: user.signature,
            _has_verified_email: Some(user.email_verified),
            _is_application_accepted: user.is_application_accepted,
            _board_creation_approved: user.board_creation_approved,
            _settings: Some(Settings {
                email: user.email,
                email_notifications_enabled: user.email_notifications_enabled,
                default_sort_type: user.default_sort_type,
                default_listing_type: user.default_listing_type,
                show_nsfw: user.show_nsfw,
                show_bots: user.show_bots,
            }),
            counts,
        }
    }
}

impl From<DbUser> for User {
    fn from(user: DbUser) -> Self {
        // Create a default UserAggregates if not provided
        let default_counts = DbUserAggregates {
            id: 0, // Default id for aggregates
            user_id: user.id,
            post_count: 0,
            post_score: 0,
            comment_count: 0,
            comment_score: 0,
        };

        Self::from((user, default_counts))
    }
}