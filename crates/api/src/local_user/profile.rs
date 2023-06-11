use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    user::{GetLoggedInUser, GetUserNamePath, Profile, ProfileResponse},
    utils::{require_user},
};
use tinyboards_db_views::structs::{LoggedInUserView, LocalUserView};
use tinyboards_utils::{error::TinyBoardsError, settings::SETTINGS};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetLoggedInUser {
    type Response = LoggedInUserView;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {

        let view = require_user(context.pool(), context.master_key(), auth)
            .await
            .unwrap()?;

        let logged_in_view = LoggedInUserView::read(context.pool(), view.person.id).await?;

        Ok(logged_in_view)
    }
}

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for Profile {
    type Response = ProfileResponse;
    type Route = GetUserNamePath;

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        route: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let rcopy = route.clone();

        let local_user_view = LocalUserView::get_by_name(context.pool(), &rcopy.username).await?;

        let settings = SETTINGS.to_owned();
        let domain = settings.hostname;
        let id = local_user_view.local_user.id;
        let avatar_url = local_user_view.person.avatar.unwrap();
        let bio = local_user_view.person.bio.unwrap_or("".to_string());
        let banner_url = local_user_view.person.banner.unwrap();
        let url = format!(
            "https://{domain}/api/v1/users/{name}",
            domain = &domain,
            name = &local_user_view.person.name
        );
        let html_url = format!(
            "https://{domain}/@{name}",
            domain = &domain,
            name = &local_user_view.person.name
        );
        let saved_url = format!(
            "https://{domain}/api/v1/users/{name}/saved",
            domain = &domain,
            name = &local_user_view.person.name
        );
        let posts_url = format!(
            "https://{domain}/api/v1/users/{name}/posts",
            domain = &domain,
            name = &local_user_view.person.name
        );
        let comments_url = format!(
            "https://{domain}/api/v1/users/{name}/comments",
            domain = &domain,
            name = &local_user_view.person.name
        );
        let mut _user_type = String::new();
        if local_user_view.local_user.is_admin {
            _user_type = String::from("Admin");
        } else {
            _user_type = String::from("User");
        }
        let is_admin = local_user_view.local_user.is_admin;
        let display_name = local_user_view.person.display_name.unwrap_or(local_user_view.person.name);

        let rcopy2 = route.clone();
        let view = LocalUserView::get_by_name(context.pool(), &rcopy2.username).await?;

        let rep = view.counts.rep;
        let posts_count = view.counts.post_count;
        let posts_score = view.counts.post_score;
        let comments_count = view.counts.comment_count;
        let comments_score = view.counts.comment_score;
        let created_at = view.local_user.creation_date;
        let updated_at = view.local_user.updated;
        let is_banned = view.local_user.is_banned;
        let is_deleted = view.local_user.is_deleted;
        let username = route.clone().username;

        Ok(ProfileResponse {
            username,
            bio,
            id,
            avatar_url,
            banner_url,
            url,
            html_url,
            saved_url,
            posts_url,
            comments_url,
            user_type: _user_type,
            is_admin,
            display_name,
            rep,
            posts_count,
            posts_score,
            comments_count,
            comments_score,
            created_at,
            updated_at,
            is_banned,
            is_deleted,
        })
    }
}
