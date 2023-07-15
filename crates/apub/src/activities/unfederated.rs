use crate::SendActivity;
use tinyboards_api_common::{
    comment::*,
    moderator::*,
    person::*,
    board::*,
    post::*,
    site::*,
    admin::*,
    applications::*, 
    data::TinyBoardsContext, 
};
use tinyboards_db::models::moderator::mod_actions::{ModBan, ModRemoveBoard};
use tinyboards_db_views::structs::{LoggedInUserView, CommentView};
use tinyboards_federation::config::Data;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait]
impl SendActivity for Register {
    type Response = SignupResponse;
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

#[async_trait::async_trait]
impl SendActivity for Profile {
    type Response = ProfileResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetLoggedInUser {
    type Response = LoggedInUserView;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetMembers {
    type Response = GetMembersResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetUserSettings {
    type Response = GetUserSettingsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for SaveUserSettings {
    type Response = LoginResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for PasswordResetRequest {
    type Response = ();
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
impl SendActivity for CreateSiteInvite {
    type Response = CreateSiteInviteResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for ValidateSiteInvite {
    type Response = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetPost {
    type Response = GetPostResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetPosts {
    type Response = GetPostsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetComment {
    type Response = CommentResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetComments {
    type Response = GetCommentsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for Login {
    type Response = LoginResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetCommentReplies {
    type Response = GetCommentRepliesResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for GetPersonMentions {
    type Response = GetPersonMentionsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for SearchNames {
    type Response = SearchNamesResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for SavePost {
    type Response = PostResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for SaveComment {
    type Response = CommentResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for BanUser {
    type Response = ModActionResponse<ModBan>;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for BanBoard {
    type Response = ModActionResponse<ModRemoveBoard>;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for DeleteFile {
    type Response = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for VerifyEmail {
    type Response = VerifyEmailResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}


#[async_trait::async_trait]
impl SendActivity for GetSiteSettings {
    type Response = GetSiteSettingsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }
}

#[async_trait::async_trait]
impl SendActivity for SaveSiteSettings {
    type Response = GetSiteSettingsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for GetUnreadCount {
    type Response = GetUnreadCountResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for MarkAllMentionsRead {
    type Response = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for MarkAllRepliesRead {
    type Response = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for ListPostReports {
    type Response = ListPostReportsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for ResolvePostReport {
    type Response = PostReportResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for ListCommentReports {
    type Response = ListCommentReportsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for ResolveCommentReport {
    type Response = CommentReportResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for GetPersonDetails {
    type Response = GetPersonDetailsResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for GetBoard {
    type Response = GetBoardResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for ResolveObject {
    type Response = ResolveObjectResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for CreateBoard {
    type Response = BoardResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for GetPostComments {
    type Response = Vec<CommentView>;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for RemoveBoard {
    type Response = BoardResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for ListSiteInvites {
    type Response = ListSiteInvitesResponse;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}

#[async_trait::async_trait]
impl SendActivity for DeleteSiteInvite {
    type Response = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> { Ok(()) }

}