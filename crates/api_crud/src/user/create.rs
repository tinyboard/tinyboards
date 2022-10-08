use crate::PerformCrud;
use actix_web::web::Data;
use porpl_api_common::data::PorplContext;
use porpl_api_common::{
    person::{LoginResponse, Register},
    sensitive::Sensitive,
    utils::blocking,
};
use porpl_db::models::user::user::{User, UserForm};
use porpl_utils::PorplError;
use regex::Regex;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for Register {
    type Response = LoginResponse;
    type Route = ();

    async fn perform(
        self,
        context: &Data<PorplContext>,
        _: Option<&str>,
    ) -> Result<LoginResponse, PorplError> {
        let data: Register = self;

        // some email verification logic here?

        // make sure site has open registration first here

        // USERNAME CHECK
        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{2,29}$").unwrap();
        if !re.is_match(&data.username) {
            return Err(PorplError::new(400, String::from("Invalid username!")));
        }

        // PASSWORD CHECK
        // password_length_check(&data.password)?;
        if !(8..60).contains(&data.password.len()) {
            return Err(PorplError::new(
                400,
                String::from("Your password must be between 8 and 60 characters long."),
            ));
        }

        // error messages here if email verification is on and no email provided, same for applicaction not being filled out

        if data.password != data.password_verify {
            return Err(PorplError::new(
                400,
                String::from("passwords do not match!"),
            ));
        }

        // captcha logic here (when we implement captcha)

        // generate apub actor_keypair here whenever we get to implementing federation

        let user_form = UserForm {
            name: data.username,
            email: data.email,
            passhash: data.password.unpack(),
            show_nsfw: Some(data.show_nsfw),
            ..UserForm::default()
        };

        let inserted_user =
            blocking(context.pool(), move |conn| User::register(conn, user_form)).await??;

        // logic about emailing the admins of the site if application submitted and email notification for user etc

        let login_response = LoginResponse {
            jwt: Some(Sensitive::new(inserted_user.get_jwt(context.master_key()))),
            registration_created: false,
            verify_email_sent: false,
        };

        // logic block about handling email verification/application messaging (hey you applied wait until admin approves)

        //login_response.jwt = inserted_user.get_jwt(context.master_key());

        Ok(login_response)
    }
}
