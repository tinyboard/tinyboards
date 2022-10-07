use crate::PerformCrud;
use actix_web::web::Data;
use porpl_db::models::user::user::{UserForm, User};
use porpl_api_common::{
    person::{LoginResponse, Register},
    utils::{
        blocking,
        password_length_check,
        get_jwt
    },
};
use porpl_api::data::PorplContext;
use porpl_utils::PorplError;


#[async_trait::async_trait(?Send)]
impl PerformCrud for Register {
    type Response = LoginResponse;

    async fn perform(
        &self,
        context: &Data<PorplContext>,
    ) -> Result<LoginResponse, PorplError> {
        let data: &Register = self;

        // some email verification logic here?

        // make sure site has open registration first here
        
        password_length_check(&data.password)?;

        // error messages here if email verification is on and no email provided, same for applicaction not being filled out

        if data.password != data.password_verify {
            return Err(PorplError::new(
                400,
                String::from("passwords do not match!")
            ))
        }

        // captcha logic here (when we implement captcha)

        // generate apub actor_keypair here whenever we get to implementing federation

        let user_form = UserForm {
            email: Some(data.email.unwrap()),
            passhash: Some(data.password.to_string()),
            show_nsfw: Some(data.show_nsfw),
            ..UserForm::default()
        };

        let inserted_user = match blocking(context.pool(), move |conn| {
            User::register(conn, &user_form)
        })
        .await?
        {
            Ok(lu) => lu,
            Err(e) => {
                eprintln!("ERROR: {e}");
                return Err(PorplError::new(500, String::from("Internal Error: failed to register user, please try again")))
            }
        };

        // logic about emailing the admins of the site if application submitted and email notification for user etc

        let mut login_response = LoginResponse {
            jwt: None,
            registration_created: false,
            verify_email_sent: false,
        };

        // logic block about handling email verification/application messaging (hey you applied wait until admin approves)

        login_response.jwt = get_jwt(inserted_user.id, inserted_user.name, context.master_key());

        Ok(login_response)
    }
}