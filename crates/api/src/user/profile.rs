use crate::Perform;
use actix_web::web::Data;
use porpl_api_common::{
    data::PorplContext,
    user::{Profile, ProfileResponse, GetUserNamePath},
    utils::{
        blocking,
    },
};
use porpl_db::{models::user::user::User};
use porpl_db_views::{
    structs::UserView,
};
use porpl_utils::{
    error::PorplError,
};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for Profile {
    type Response = ProfileResponse;
    type Route = GetUserNamePath;

    async fn perform(
        self,
        context: &Data<PorplContext>,
        route: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, PorplError> {

        let rcopy = route.clone();

        let user = blocking(context.pool(), move |conn| {
            User::get_by_name(conn, &rcopy.username)
        })
        .await?
        .map_err(|_| PorplError::from_string("user not found", 404))?;
        
        let domain = std::env::var("MASTER_DOMAIN").unwrap();
        let id = user.id;
        let avatar_url = user.avatar.unwrap_or("".to_string());
        let url = format!("https://{domain}/api/v1/users/{name}", domain=&domain, name=&user.name);
        let html_url = format!("https://{domain}/@{name}", domain=&domain, name=&user.name);
        let saved_url = format!("https://{domain}/api/v1/users/{name}/saved", domain=&domain, name=&user.name);
        let posts_url = format!("https://{domain}/api/v1/users/{name}/posts", domain=&domain, name=&user.name);
        let comments_url = format!("https://{domain}/api/v1/users/{name}/comments", domain=&domain, name=&user.name);
        let mut _user_type = String::new();
        if user.admin {
            _user_type = String::from("Admin");
        } else {
            _user_type = String::from("User");
        }
        let is_admin = user.admin;
        let display_name = user.preferred_name.unwrap_or(user.name);

        let rcopy2 = route.clone();
        let view 
            = blocking(context.pool(), move |conn| {
                UserView::read_from_name(conn, &rcopy2.username)
                    .map_err(|_| PorplError::err_500())
            })
            .await??;
        
        let posts_count = view.counts.post_count;
        let posts_score = view.counts.post_score;
        let comments_count = view.counts.comment_count;
        let comments_score = view.counts.comment_score;
        let created_at = user.published;
        let updated_at = user.updated;
        let is_banned = user.banned;
        let is_deleted = user.deleted;
        let username = route.clone().username;

        Ok( ProfileResponse {
            username,
            id,
            avatar_url,
            url,
            html_url,
            saved_url,
            posts_url,
            comments_url,
            user_type: _user_type,
            is_admin,
            display_name,
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
