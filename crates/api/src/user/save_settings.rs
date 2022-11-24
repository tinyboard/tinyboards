use crate::Perform;
use actix_web::web::Data;
use tinyboards_api_common::{
    user::{LoginResponse, SaveUserSettings},
    utils::{get_user_view_from_jwt, blocking},
    data::TinyBoardsContext, sensitive::Sensitive
};
use tinyboards_db::{
    models::user::user::{User, UserForm},
    models::site::site::Site,
    traits::Crud,
    utils::{
        diesel_option_overwrite, 
        naive_now,
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
            })
            .await??;
            

        let avatar = diesel_option_overwrite(&data.avatar);
        let banner = diesel_option_overwrite(&data.banner);
        let bio = diesel_option_overwrite(&data.bio);
        let email = data.email.as_deref().map(str::to_lowercase);


        // UNCOMMENT THIS WHEN WE HAVE NOTIFICATION EMAIL LOGIC DONE (and send email notification to the user if email is changed) 

        // if let Some(Some(email)) = &email {
        //     let previous_email = user_view.user.email.unwrap_or_default();
        //     // send email notification if email was changed
        //     if previous_email.ne(email) {
        //         // do notification email logic here
        //     }
        // }

        
        if email.is_none() && site.email_verification_required {
            return Err(TinyBoardsError::from_message("email required"));
        }
        
        if let Some(Some(bio)) = &bio {
            // seems sort of arbitrary? do we want a setting for this length somewhere?
            if bio.chars().count() > 300 {
                return Err(TinyBoardsError::from_message("bio too long"));
            }
        }

        let default_listing_type = data.default_listing_type;
        let default_sort_type = data.default_sort_type;
        // grabbing the current timestamp for the update
        let updated = Some(naive_now());

        let user_form = UserForm {
            bio,
            email,
            show_nsfw: data.show_nsfw,
            theme: data.theme.clone(),
            avatar,
            banner,
            default_listing_type,
            default_sort_type,
            updated,
            ..UserForm::default()
        };

        // perform settings update
        blocking(context.pool(), move |conn| {
            User::update(conn, user_view.user.id, &user_form)
                .map_err(|_| TinyBoardsError::from_message("could not update user settings"))
        })
        .await??;

        let updated_user_view = 
            get_user_view_from_jwt(auth, context.pool(), context.master_key()).await?;

        let new_jwt = Claims::jwt(
                updated_user_view.user.id,
                &context.master_key().jwt,
                &context.settings().hostname,
            )?;

        // return the jwt
        Ok(LoginResponse {
            jwt: Sensitive::new(new_jwt), 
            user: updated_user_view 
        })
    }
}