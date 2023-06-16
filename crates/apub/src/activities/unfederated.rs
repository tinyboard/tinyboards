use crate::SendActivity;
use tinyboards_api_common::{
    comment::{
        CommentResponse,
        GetComment,
        ListComments,
        ListCommentsResponse,
        SaveComment,
        // ListCommentReports,
        // ListCommentReportsResponse,
        // ResolveCommentReport,
        // CommentReportResponse,
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
      },
};

impl SendActivity for Register {
    type Response = LoginResponse;
}

impl SendActivity for GetFederatedInstances {
    type Response = GetFederatedInstancesResponse;
}

impl SendActivity for PurgeBoard {
    type Response = PurgeItemResponse;
}

impl SendActivity for PurgePost {
    type Response = PurgeItemResponse;
}

impl SendActivity for PurgeComment {
    type Response = PurgeItemResponse;
}

impl SendActivity for PurgePerson {
    type Response = PurgeItemResponse;
}

impl SendActivity for HandleRegistrationApplication {
    type Response = HandleRegistrationApplicationResponse;
}

impl SendActivity for ListRegistrationApplications {
    type Response = ListRegistrationApplicationsResponse;
}

impl SendActivity for Search {
    type Response = SearchResponse;
}

impl SendActivity for ExecutePasswordReset {
    type Response = ExecutePasswordResetResponse;
}

impl SendActivity for AddAdmin {
    type Response = AddAdminResponse;
}

impl SendActivity for LeaveAdmin {
    type Response = GetSiteResponse;
}

impl SendActivity for GetSite {
    type Response = GetSiteResponse;
}

impl SendActivity for ListComments {
    type Response = ListCommentsResponse;
}