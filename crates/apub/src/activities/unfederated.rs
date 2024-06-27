use crate::SendActivity;
use tinyboards_api_common::{
    admin::*,
    applications::*,
    board::*,
    comment::*,
    data::TinyBoardsContext,
    emoji::*,
    message::{GetMessages, GetMessagesResponse},
    moderator::*,
    person::*,
    post::*,
    site::*,
};
use tinyboards_db::models::moderator::mod_actions::{ModBan, ModRemoveBoard};
use tinyboards_db_views::structs::{CommentView, LoggedInUserView};
use tinyboards_federation::config::Data;
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait]
impl SendActivity for Register {
    type Response = SignupResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetFederatedInstances {
    type Response = GetFederatedInstancesResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for PurgeBoard {
    type Response = PurgeItemResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for PurgePost {
    type Response = PurgeItemResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for PurgeComment {
    type Response = PurgeItemResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for PurgePerson {
    type Response = PurgeItemResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for HandleRegistrationApplication {
    type Response = HandleRegistrationApplicationResponse;
    type Route = ApplicationIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListRegistrationApplications {
    type Response = ListRegistrationApplicationsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for Search {
    type Response = SearchResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for AddAdmin {
    type Response = AddAdminResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for InviteBoardMod {
    type Response = BoardModResponse;
    type Route = BoardIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for LeaveAdmin {
    type Response = GetSiteResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetSite {
    type Response = GetSiteResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListBoardMods {
    type Response = ListBoardModsResponse;
    type Route = BoardIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListComments {
    type Response = ListCommentsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for Profile {
    type Response = ProfileResponse;
    type Route = GetUserNamePath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetLoggedInUser {
    type Response = LoggedInUserView;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetMembers {
    type Response = GetMembersResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetUserSettings {
    type Response = GetUserSettingsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for SaveUserSettings {
    type Response = LoginResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for PasswordResetRequest {
    type Response = ();
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ExecutePasswordReset {
    type Response = ExecutePasswordResetResponse;
    type Route = PasswordResetTokenPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for CreateSiteInvite {
    type Response = CreateSiteInviteResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ValidateSiteInvite {
    type Response = ();
    type Route = InviteToken;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetPost {
    type Response = GetPostResponse;
    type Route = PostIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetPosts {
    type Response = GetPostsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for PostModQueue {
    type Response = GetPostsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for CommentModQueue {
    type Response = GetCommentsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetPostReports {
    type Response = ListPostReportsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetComment {
    type Response = ListCommentsResponse;
    type Route = CommentIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetComments {
    type Response = GetCommentsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetCommentReports {
    type Response = ListCommentReportsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for Login {
    type Response = LoginResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetCommentReplies {
    type Response = GetCommentRepliesResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetPersonMentions {
    type Response = GetPersonMentionsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetMessages {
    type Response = GetMessagesResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for SearchNames {
    type Response = SearchNamesResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for SavePost {
    type Response = PostResponse;
    type Route = PostIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for SaveComment {
    type Response = CommentResponse;
    type Route = CommentIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ToggleBan {
    type Response = ModActionResponse<ModBan>;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for CheckBoardExists {
    type Response = BoardExistsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for BanBoard {
    type Response = ModActionResponse<ModRemoveBoard>;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for DeleteFile {
    type Response = ();
    type Route = FileNamePath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for VerifyEmail {
    type Response = VerifyEmailResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetSiteSettings {
    type Response = GetSiteSettingsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for SaveSiteSettings {
    type Response = GetSiteSettingsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetUnreadCount {
    type Response = GetUnreadCountResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for MarkAllMentionsRead {
    type Response = ();
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for MarkAllMessagesRead {
    type Response = ();
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for MarkAllRepliesRead {
    type Response = ();
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListPostReports {
    type Response = ListPostReportsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ResolvePostReport {
    type Response = PostReportResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListCommentReports {
    type Response = ListCommentReportsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ResolveCommentReport {
    type Response = CommentReportResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetPersonDetails {
    type Response = GetPersonDetailsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetBoard {
    type Response = GetBoardResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ResolveObject {
    type Response = ResolveObjectResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for CreateBoard {
    type Response = BoardResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for GetPostComments {
    type Response = Vec<CommentView>;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for RemoveBoard {
    type Response = BoardResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListSiteInvites {
    type Response = ListSiteInvitesResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for DeleteSiteInvite {
    type Response = ();
    type Route = InviteId;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for CreateEmoji {
    type Response = EmojiResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for EditEmoji {
    type Response = EmojiResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for DeleteEmoji {
    type Response = DeleteEmojiResponse;
    type Route = EmojiIdPath;
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SendActivity for ListBannedPersons {
    type Response = ListBannedPersonsResponse;
    type Route = ();
    async fn send_activity(
        _request: &Self,
        _response: &Self::Response,
        _context: &Data<TinyBoardsContext>,
        _path: &Self::Route,
        _auth: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        Ok(())
    }
}
