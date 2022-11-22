use crate::PerformCrud;
use actix_web::web::Data;
use regex::Regex;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::{
    sensitive::Sensitive,
    user::{Register, SignupResponse},
    utils::blocking,
};
use tinyboards_db::models::user::user::{User, UserForm};
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

        // some email verification logic here?

        // make sure site has open registration first here

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

        // error messages here if email verification is on and no email provided, same for applicaction not being filled out

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

        let inserted_user =
            blocking(context.pool(), move |conn| User::register(conn, user_form)).await??;

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
