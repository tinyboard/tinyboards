use crate::PerformCrud;
use tinyboards_ap_federation::http_signatures::generate_actor_keypair;
use actix_web::web::Data;
use regex::Regex;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::utils::send_new_applicant_email_to_admins;
use tinyboards_api_common::{
    sensitive::Sensitive,
    user::{Register, SignupResponse},
    utils::{send_verification_email, generate_inbox_url, generate_shared_inbox_url, generate_local_apud_endpoint, EndpointType},
};
use tinyboards_db::models::person::person::PersonForm;
use tinyboards_db::models::site::registration_applications::{RegistrationApplicationForm, RegistrationApplication};
use tinyboards_db::models::site::site::Site;
use tinyboards_db::models::site::site_invite::SiteInvite;
use tinyboards_db::models::person::local_user::*;
use tinyboards_db::traits::Crud;
use tinyboards_utils::TinyBoardsError;
//use tinyboards_utils::utils::generate_rand_string;

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

        let site = Site::read_local(context.pool()).await?;

        // some email verification logic here?
        if !site.open_registration && site.invite_only && data.invite_token.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "invite is required for registration",
            ));
        }

        if !site.open_registration && site.require_application && data.answer.is_none() {
            return Err(TinyBoardsError::from_message(
                403, 
                "application answer is required"));
        }
        
        // USERNAME CHECK
        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{2,29}$").unwrap();
        if !re.is_match(&data.username) {
            return Err(TinyBoardsError::from_message(400, "invalid username"));
        }

        // PASSWORD CHECK
        // password_length_check(&data.password)?;
        if !(8..60).contains(&data.password.len()) {
            return Err(TinyBoardsError::from_message(
                400,
                "Your password must be between 8 and 60 characters long.",
            ));
        }

        if site.email_verification_required && data.email.is_none() {
            return Err(TinyBoardsError::from_message(
                400,
                "email verification is required, please provide an email",
            ));
        }

        // captcha logic here (when we implement captcha)

        let actor_keypair = generate_actor_keypair()?;

        let actor_id = generate_local_apud_endpoint(
            EndpointType::Person, 
            &data.username, 
            &context.settings().get_protocol_and_hostname()
        )?;

        // now we need to create both a local_user and a person (for apub)
        let person_form = PersonForm {
            name: Some(data.username.clone()),
            actor_id: Some(actor_id.to_string().clone()),
            private_key: Some(Some(actor_keypair.private_key)),
            public_key: Some(Some(actor_keypair.public_key)),
            inbox_url: Some(generate_inbox_url(&actor_id)?.to_string()),
            shared_inbox_url: Some(Some(generate_shared_inbox_url(&actor_id)?.to_string())),
            ..PersonForm::default()
            // todo - add instance_id in later
        };

        let user_form = LocalUserForm {
            name: Some(data.username.clone()),
            email: Some(data.email),
            passhash: Some(data.password.unpack()),
            ..LocalUserForm::default()
        };

        let mut invite = None;

        // perform a quick check if the site is in invite_only mode to see if the invite_token is valid
        if site.invite_only {
            invite = Some(SiteInvite::read_for_token(context.pool(), &invite_token.unwrap()).await?); // (if the invite token is valid there will be a entry in the db for it)
        }

        let inserted_user = LocalUser::register(context.pool(), user_form).await?;

        // if the user was invited, invalidate the invite token here by removing from db
        if site.invite_only {
            SiteInvite::delete(context.pool(), invite.unwrap().id).await?;
        }

        // if site is in application mode, add the application to the database
        if site.require_application {
            let form = RegistrationApplicationForm {
                person_id: inserted_user.id,
                answer: data.answer.clone(),
                ..RegistrationApplicationForm::default()
            };

            RegistrationApplication::create(context.pool(), &form).await?;
        }

        // email the admins regarding the new application
        if site.require_application {
            send_new_applicant_email_to_admins(&data.username, context.pool(), context.settings())
            .await?;
        }

        let email = inserted_user.email.clone().unwrap_or_default();

        // send a verification email if email verification is required
        if site.email_verification_required {
            send_verification_email(&inserted_user, &email, context.pool(), context.settings())
                .await?;
        }

        let mut response = SignupResponse {
            jwt: Some(Sensitive::new(
                inserted_user.get_jwt(context.master_key().jwt.as_ref()),
            )),
            registration_created: false,
            verify_email_sent: false,
        };

        if site.require_application {
            response.registration_created = true;
            response.jwt = None;
        }

        // logic block about handling email verification/application messaging (hey you applied wait until admin approves)

        //login_response.jwt = inserted_user.get_jwt(context.master_key());

        Ok(response)
    }
}
