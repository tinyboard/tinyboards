use crate::PerformCrud;
use actix_web::web::Data;
use regex::Regex;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::{
    sensitive::Sensitive,
    user::{Register, SignupResponse},
    utils::{blocking, send_verification_email},
};
use tinyboards_db::models::site::site_invite::SiteInvite;
use tinyboards_db::traits::Crud;
use tinyboards_db::{
    models::{
        user::user::{User, UserForm},
        site::site::Site,
    },
};
use tinyboards_utils::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for Register {
    type Response = SignupResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        _: Option<&str>,
    ) -> Result<Self::Response, TinyBoardsError> {
        let data: Register = self;

        let invite_token = data.invite_token.clone();

        let site= blocking(context.pool(), move |conn| {
            Site::read_local(conn)
        })
        .await??;

        // some email verification logic here?

        // make sure site has open registration first here
        if !site.open_registration  {
            return Err(TinyBoardsError::from_message("site is not in open registration mode"))
        }

        if !site.open_registration && site.invite_only && data.invite_token.is_none() {
            return Err(TinyBoardsError::from_message("invite is required for registration"))
        }

        // USERNAME CHECK
        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{2,29}$").unwrap();
        if !re.is_match(&data.username) {
            return Err(TinyBoardsError::from_message("invalid username"));
        }

        // PASSWORD CHECK
        // password_length_check(&data.password)?;
        if !(8..60).contains(&data.password.len()) {
            return Err(TinyBoardsError::from_message("Your password must be between 8 and 60 characters long."));
        }

        if site.email_verification_required && data.email.is_none() {
            return Err(TinyBoardsError::from_message("email verification is required, please provide an email"));
        }

        /*if data.password != data.password_verify {
            return Err(TinyBoardsError::new(
                400,
                String::from("passwords do not match!"),
            ));
        }*/

        // captcha logic here (when we implement captcha)

        // generate apub actor_keypair here whenever we get to implementing federation

        let user_form = UserForm {
            name: data.username.clone(),
            email: data.email,
            passhash: data.password.unpack(),
            ..UserForm::default()
        };

        let mut invite = None;

        // perform a quick check if the site is in invite_only mode to see if the invite_token is valid
        if site.invite_only {
            invite = Some(blocking(context.pool(), move |conn| {
                SiteInvite::read_for_token(conn, &invite_token.unwrap())
            })
            .await??); // (if the invite token is valid there will be a entry in the db for it)
        }

        let inserted_user =
            blocking(context.pool(), move |conn| User::register(conn, user_form)).await??;

        // if the user was invited, invalidate the invite token here by removing from db
        if site.invite_only {
            blocking(context.pool(), move |conn| SiteInvite::delete(conn, invite.unwrap().id))
                .await??;
        }

        let email = inserted_user.email.clone().unwrap();

        // send a verification email if email verification is required
        if site.email_verification_required {
            send_verification_email(&inserted_user, &email, context.pool(), context.settings()).await?;
        }

        // logic about emailing the admins of the site if application submitted and email notification for user etc

        let response = SignupResponse {
            jwt: Some(Sensitive::new(
                inserted_user.get_jwt(context.master_key().jwt.as_ref()),
            )),
            registration_created: false,
            verify_email_sent: false,
        };

        // logic block about handling email verification/application messaging (hey you applied wait until admin approves)

        //login_response.jwt = inserted_user.get_jwt(context.master_key());

        Ok(response)
    }
}
