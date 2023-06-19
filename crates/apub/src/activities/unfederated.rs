use crate::SendActivity;
use tinyboards_api_common::{
    comment::{
        CommentResponse,
        GetComment,
        ListComments,
        ListCommentsResponse,
        SaveComment,
        ListCommentReports,
        ListCommentReportsResponse,
        ResolveCommentReport,
        CommentReportResponse,
    },
    board::{
        BoardResponse,
        CreateBoard,
        // GetBoard,
        // GetBoardResponse,
        // ListBoards,
        // ListBoardsResponse,
        // TransferBoard,
    },
    // custom_emoji::{
    //     CreateCustomEmoji,
    //     CustomEmojiResponse,
    //     DeleteCustomEmoji,
    //     DeleteCustomEmojiResponse,
    //     EditCustomEmoji,
    // },
    person::{
        // BannedPersonsResponse,
        // BlockPerson,
        // BlockPersonResponse,
        ChangePassword,
        // CommentReplyResponse,
        // GetBannedPersons,
        // GetCaptcha,
        // GetCaptchaResponse,
        // GetPersonDetails,
        // GetPersonDetailsResponse,
        // GetPersonMentions,
        // GetPersonMentionsResponse,
        // GetReportCount,
        // GetReportCountResponse,
        GetUnreadCount,
        GetUnreadCountResponse,
        Login,
        LoginResponse,
        // MarkAllAsRead,
        // MarkCommentReplyAsRead,
        // MarkPersonMentionAsRead,
        // PersonMentionResponse,
        Register,
        SaveUserSettings,
        VerifyEmail,
        VerifyEmailResponse,
    },
    post::{
        GetPost,
        GetPostResponse,
        ListPosts,
        ListPostsResponse,
        PostResponse,
        SavePost,
        // GetSiteMetadata,
        // GetSiteMetadataResponse,
        // ListPostReports,
        // ListPostReportsResponse,
        // MarkPostAsRead,
        // PostReportResponse,
        // ResolvePostReport,
      },
      site::{
        // ApproveRegistrationApplication,
        // CreateSite,
        // EditSite,
        GetFederatedInstances,
        GetFederatedInstancesResponse,
        ExecutePasswordReset,
        ExecutePasswordResetResponse,
        // GetModlog,
        // GetModlogResponse,
        GetSite,
        GetSiteResponse,
        // RegistrationApplicationResponse,
        // ResolveObject,
        // ResolveObjectResponse,
        Search,
        SearchResponse,
        SiteResponse,
      },
      admin::{
        AddAdmin,
        AddAdminResponse,
        LeaveAdmin,
        PurgeBoard,
        PurgeComment,
        PurgePost,
        PurgePerson,
        PurgeItemResponse,
        HandleRegistrationApplication,
        HandleRegistrationApplicationResponse,
      },
      applications::{
        ListRegistrationApplications,
        ListRegistrationApplicationsResponse,
      }, data::TinyBoardsContext,
};
use tinyboards_federation::config::Data;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait]
impl SendActivity for Register {
    type Response = LoginResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetFederatedInstances {
    type Response = GetFederatedInstancesResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for PurgeBoard {
    type Response = PurgeItemResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for PurgePost {
    type Response = PurgeItemResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for PurgeComment {
    type Response = PurgeItemResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for PurgePerson {
    type Response = PurgeItemResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for HandleRegistrationApplication {
    type Response = HandleRegistrationApplicationResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for ListRegistrationApplications {
    type Response = ListRegistrationApplicationsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for Search {
    type Response = SearchResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for ExecutePasswordReset {
    type Response = ExecutePasswordResetResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for AddAdmin {
    type Response = AddAdminResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for LeaveAdmin {
    type Response = GetSiteResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetSite {
    type Response = GetSiteResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for ListComments {
    type Response = ListCommentsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}