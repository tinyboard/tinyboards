use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    person::{GetLoggedInUser, GetUserNamePath, Profile, ProfileResponse},
    utils::require_user, request::fetch_remote_user,
};
use tinyboards_db_views::structs::{LoggedInUserView, LocalUserView, PersonView};
use tinyboards_utils::{error::TinyBoardsError, settings::SETTINGS};
use url::Url;

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

        let username = &rcopy.username;
        let is_local_user = LocalUserView::get_by_name(context.pool(), username).await.is_ok();

        if !is_local_user && username.contains("@") {
            let fetch_url = format!("{}/api/v1/user?username={}", context.settings().get_protocol_and_hostname(), username);
            let _response = fetch_remote_user(context.client(), &Url::parse(&fetch_url)?).await?;
        } else {
            return Err(TinyBoardsError::from_message(404, "username not found in local database"));
        }

        let person_view = PersonView::read_from_name(context.pool(), username).await?;

        //let local_user_view = LocalUserView::get_by_name(context.pool(), username).await?;

        let settings = SETTINGS.to_owned();
        let domain = settings.hostname;
        let id = person_view.person.id;
        let avatar_url = person_view.person.avatar;
        let bio = person_view.person.bio.unwrap_or("".to_string());
        let banner_url = person_view.person.banner;
        let url = format!(
            "https://{domain}/api/v1/users/{name}",
            domain = &domain,
            name = &person_view.person.name
        );
        let html_url = format!(
            "https://{domain}/@{name}",
            domain = &domain,
            name = &person_view.person.name
        );
        let saved_url = format!(
            "https://{domain}/api/v1/users/{name}/saved",
            domain = &domain,
            name = &person_view.person.name
        );
        let posts_url = format!(
            "https://{domain}/api/v1/users/{name}/posts",
            domain = &domain,
            name = &person_view.person.name
        );
        let comments_url = format!(
            "https://{domain}/api/v1/users/{name}/comments",
            domain = &domain,
            name = &person_view.person.name
        );
        let mut _user_type = String::new();

        if person_view.person.is_admin {
            _user_type = String::from("Admin");
        } else {
            _user_type = String::from("User");
        }

        let is_admin = person_view.person.is_admin;
        let display_name = person_view.person.display_name.unwrap_or(person_view.person.name);

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
