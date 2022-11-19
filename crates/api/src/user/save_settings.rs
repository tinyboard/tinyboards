use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    user::{LoginResponse, SaveUserSettings},
    utils::{get_user_view_from_jwt, blocking},
    data::TinyBoardsContext
};
use tinyboards_db::{
    models::user::user::{User, UserForm},
    models::site::site::Site,
    traits::Crud,
    utils::{
        diesel_option_overwrite_to_url,
        diesel_option_overwrite,
    },
};
use tinyboards_utils::{
    error::TinyBoardsError,
    claims::Claims,
};

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for SaveUserSettings {
    type Response = LoginResponse;
    type Route = ();

    #[tracing::instrument(skip(context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>
    ) -> Result<LoginResponse, TinyBoardsError> {

        let data: &SaveUserSettings = &self;

        let user_view =
            get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let site =
            blocking(context.pool(), move |conn| {
                Site::read_local(conn)
                    .map_err(|_| TinyBoardsError::from_string("could not read local site", 500))
            })
            .await??;
            

        let avatar = diesel_option_overwrite_to_url(&data.avatar)?;
        let banner = diesel_option_overwrite_to_url(&data.banner)?;
        let bio = diesel_option_overwrite(&data.bio);
        let email_dereffed = data.email.as_deref().map(str::to_lowercase);
        let email = diesel_option_overwrite(&email_dereffed);

        // if let Some(Some(email)) = &email {
        //     let previous_email = user_view.user.email.unwrap_or_default();
        //     // send email notification if email was changed
        //     if previous_email.ne(email) {
        //         // do notification email logic here
        //     }
        // }

        if let Some(email) = &email {
            if email.is_none() && site.email_verification_required {
                return Err(TinyBoardsError::from_string("email required", 500));
            }
        }

        if let Some(Some(bio)) = &bio {
            // seems sort of arbitrary? do we want a setting for this length somewhere?
            if bio.chars().count() > 300 {
                return Err(TinyBoardsError::from_string("bio too long", 500));
            }
        }

        let default_listing_type = data.default_listing_type;
        let default_sort_type = data.default_sort_type;

        let user_form = UserForm {
            bio,
            email,
            show_nsfw: data.show_nsfw,
            theme: data.theme,
            avatar,
            banner,
            default_listing_type,
            default_sort_type,
            ..UserForm::default()
        };

        // perform settings update
        blocking(context.pool(), move |conn| {
            User::update(conn, user_view.user.id, &user_form)
                .map_err(|_| TinyBoardsError::from_string("could not update user settings", 500))
        })
        .await??;

        let updated_user_view = 
            get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        // return the jwt
        Ok(LoginResponse { 
            jwt: Some(
                Claims::jwt(
                    updated_user_view.user.id,
                    &context.master_key(),
                    &context.settings().hostname,
                )?
                .into()
            ), 
            user: updated_user_view 
        })
    }
}