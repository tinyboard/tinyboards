use crate::SendActivity;
use tinyboards_api_common::{
    comment::{
        // CommentReportResponse,
        CommentResponse,
        GetComment,
        ListComments,
        ListCommentsResponse,
        // ListCommentReports,
        // ListCommentReportsResponse,
        // ResolveCommentReport,
        SaveComment,
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
        // AddAdmin,
        // AddAdminResponse,
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
        // PasswordChangeAfterReset,
        // PasswordReset,
        // PasswordResetResponse,
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
        // GetSiteMetadata,
        // GetSiteMetadataResponse,
        // ListPostReports,
        // ListPostReportsResponse,
        // MarkPostAsRead,
        // PostReportResponse,
        PostResponse,
        // ResolvePostReport,
        SavePost,
      },
      site::{
        // ApproveRegistrationApplication,
        // CreateSite,
        // EditSite,
        GetFederatedInstances,
        GetFederatedInstancesResponse,
        // GetModlog,
        // GetModlogResponse,
        // GetSite,
        // GetSiteResponse,
        // LeaveAdmin,
        // RegistrationApplicationResponse,
        // ResolveObject,
        // ResolveObjectResponse,
        Search,
        SearchResponse,
        // SiteResponse,
      },
      admin::{
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
      }
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