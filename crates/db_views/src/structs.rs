use serde::{Deserialize, Serialize};
use tinyboards_db::{
    aggregates::structs::{
        BoardAggregates, CommentAggregates, PostAggregates, SiteAggregates, PersonAggregates,
    },
    models::{
        board::boards::{BoardSafe, Board},
        comment::{comments::Comment, comment_reply::CommentReply, comment_report::CommentReport},
        post::{posts::Post, post_report::PostReport},
        site::{site::Site, site_invite::SiteInvite, registration_applications::RegistrationApplication, local_site::LocalSite, local_site_rate_limit::LocalSiteRateLimit},
        person::{
            person_mentions::*,
            local_user::*,
            person::*,
        },
    },
    SubscribedType,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalUserView {
    pub local_user: LocalUser,
    pub person: Person,
    pub counts: PersonAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonView {
    pub person: PersonSafe,
    pub settings: Option<LocalUserSettings>,
    pub counts: PersonAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggedInUserView {
    pub person: PersonSafe,
    pub settings: Option<LocalUserSettings>,
    pub counts: PersonAggregates,
    pub unread_notifications: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalUserSettingsView {
    pub settings: LocalUserSettings,
    pub counts: PersonAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PostView {
    pub post: Post,
    pub creator: Option<PersonSafe>,
    pub board: BoardSafe,
    pub creator_banned_from_board: bool, // Left Join BoardUserBan
    pub counts: PostAggregates,
    pub subscribed: SubscribedType, // Left Join BoardSubscriber
    pub saved: bool,                // Left join PostSaved
    pub read: bool,                 // Left join PostRead
    pub creator_blocked: bool,      // Left join UserBlock
    pub my_vote: Option<i16>,       // Left join PostLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentView {
    pub comment: Comment,
    pub creator: Option<PersonSafe>,
    pub post: Post,
    pub board: BoardSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool,
    pub subscribed: SubscribedType,
    pub saved: bool,
    pub creator_blocked: bool,
    pub my_vote: Option<i16>,
    pub replies: Vec<CommentView>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardBlockView {
    pub user: PersonSafe,
    pub board: BoardSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardSubscriberView {
    pub board: Board,
    pub subscriber: Person,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardModeratorView {
    pub board: BoardSafe,
    pub moderator: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardPersonBanView {
    pub board: BoardSafe,
    pub user: PersonSafe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardView {
    pub board: BoardSafe,
    pub subscribed: SubscribedType,
    pub blocked: bool,
    pub counts: BoardAggregates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserBlockView {
    pub person: PersonSafe,
    pub target: PersonSafe,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PersonMentionView {
    pub person_mention: PersonMention,
    pub comment: Comment,
    pub creator: PersonSafe,
    pub post: Post,
    pub board: BoardSafe,
    pub recipient: PersonSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool, // Left join BoardUserBan
    pub subscribed: SubscribedType,      // Left join to BoardSubscriber
    pub saved: bool,                     // Left join to CommentSaved
    pub creator_blocked: bool,           // Left join to UserBlock
    pub my_vote: Option<i16>,            // Left join to CommentLike
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteInviteView {
    pub invite: SiteInvite,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiteView {
    pub site: Site,
    pub local_site: LocalSite,
    pub local_site_rate_limit: LocalSiteRateLimit,
    pub counts: SiteAggregates,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommentReplyView {
    pub comment_reply: CommentReply,
    pub comment: Comment,
    pub creator: PersonSafe,
    pub post: Post,
    pub board: BoardSafe,
    pub recipient: PersonSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool, // Left Join to BoardUserBan
    pub subscribed: SubscribedType,      // Left join to BoardSubscribers
    pub saved: bool,                     // Left join to CommentSaved
    pub creator_blocked: bool,           // Left join to PersonBlock
    pub my_vote: Option<i16>,            // Left join to CommentLike
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct RegistrationApplicationView {
  pub application: RegistrationApplication,
  pub applicant_settings: LocalUserSettings,
  pub applicant: LocalUserSafe,
  pub admin: Option<LocalUserSafe>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PostReportView {
    pub post_report: PostReport,
    pub post: Post,
    pub board: Board,
    pub creator: PersonSafe,
    pub post_creator: PersonSafe,
    pub creator_banned_from_board: bool,
    pub my_vote: Option<i16>,
    pub counts: PostAggregates,
    pub resolver: Option<PersonSafe>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct CommentReportView {
    pub comment_report: CommentReport,
    pub comment: Comment,
    pub post: Post,
    pub board: Board,
    pub creator: PersonSafe,
    pub comment_creator: PersonSafe,
    pub counts: CommentAggregates,
    pub creator_banned_from_board: bool,
    pub my_vote: Option<i16>,
    pub resolver: Option<PersonSafe>,
}