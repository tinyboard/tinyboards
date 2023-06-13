use crate::PerformCrud;
use tinyboards_db_views::structs::SiteView;
use tinyboards_federation::http_signatures::generate_actor_keypair;
use actix_web::web::Data;
use regex::Regex;
use tinyboards_api_common::data::TinyBoardsContext;
use tinyboards_api_common::utils::send_new_applicant_email_to_admins;
use tinyboards_api_common::{
    sensitive::Sensitive,
    user::{Register, SignupResponse},
    utils::{send_verification_email, generate_inbox_url, generate_shared_inbox_url, generate_local_apub_endpoint, EndpointType},
};
use tinyboards_db::models::person::person::*;
use tinyboards_db::models::site::registration_applications::{RegistrationApplicationForm, RegistrationApplication};
use tinyboards_db::models::site::site_invite::SiteInvite;
use tinyboards_db::models::person::local_user::*;
use tinyboards_db::traits::Crud;
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

        let site_view = SiteView::read_local(context.pool()).await?;
        let local_site = site_view.local_site.clone();

        // some email verification logic here?
        if !local_site.open_registration && local_site.invite_only && data.invite_token.is_none() {
            return Err(TinyBoardsError::from_message(
                403,
                "invite is required for registration",
            ));
        }

        if !local_site.open_registration && local_site.require_application && data.answer.is_none() {
            return Err(TinyBoardsError::from_message(
                403, 
                "application answer is required"
            ));
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

        if local_site.require_email_verification && data.email.is_none() {
            return Err(TinyBoardsError::from_message(
                400,
                "email verification is required, please provide an email",
            ));
        }

        // captcha logic here (when we implement captcha)

        let actor_keypair = generate_actor_keypair()?;

        let actor_id = generate_local_apub_endpoint(
            EndpointType::Person, 
            &data.username, 
            &context.settings().get_protocol_and_hostname()
        )?;

        // now we need to create both a local_user and a person (for apub)
        let person_form = PersonForm {
            name: Some(data.username.clone()),
            actor_id: Some(actor_id.clone()),
            private_key: Some(actor_keypair.private_key),
            public_key: Some(actor_keypair.public_key),
            inbox_url: Some(generate_inbox_url(&actor_id)?),
            shared_inbox_url: Some(generate_shared_inbox_url(&actor_id)?),
            instance_id: Some(site_view.site.instance_id),
            ..PersonForm::default()
        };

        let person = Person::create(context.pool(), &person_form).await?;


        let local_user_form = LocalUserForm {
            name: Some(data.username.clone()),
            email: Some(data.email),
            passhash: Some(data.password.unpack()),
            ..LocalUserForm::default()
        };

        let mut invite = None;

        // perform a quick check if the site is in invite_only mode to see if the invite_token is valid
        if local_site.invite_only {
            invite = Some(SiteInvite::read_for_token(context.pool(), &invite_token.unwrap()).await?); // (if the invite token is valid there will be a entry in the db for it)
        }

        let inserted_local_user = match LocalUser::create(context.pool(), &local_user_form).await {
            Ok(lu) => lu,
            Err(e) => {
                let err_type = if e.to_string() == "duplicate key value violates unique constraint \"local_user_email_key\"" {
                    "email_already_exists"
                } else {
                    "user_already_exists"
                };

                // if local_user creation failed then delete the person
                Person::delete(context.pool(), person.id).await?;

                return Err(TinyBoardsError::from_error_message(e, 500, err_type));
            }
        };

        //let inserted_user = LocalUser::register(context.pool(), user_form).await?;

        // if the user was invited, invalidate the invite token here by removing from db
        if local_site.invite_only {
            SiteInvite::delete(context.pool(), invite.unwrap().id).await?;
        }

        // if site is in application mode, add the application to the database
        if local_site.require_application {
            let form = RegistrationApplicationForm {
                person_id: person.id,
                answer: data.answer.clone(),
                ..RegistrationApplicationForm::default()
            };

            RegistrationApplication::create(context.pool(), &form).await?;
        }

        // email the admins regarding the new application
        if local_site.require_application {
            send_new_applicant_email_to_admins(&data.username, context.pool(), context.settings())
            .await?;
        }

        let email = inserted_local_user.email.clone().unwrap_or_default();

        // send a verification email if email verification is required
        if local_site.require_email_verification {
            send_verification_email(&inserted_local_user, &email, context.pool(), context.settings())
                .await?;
        }

        let mut response = SignupResponse {
            jwt: Some(Sensitive::new(
                inserted_local_user.get_jwt(context.master_key().jwt.as_ref()),
            )),
            registration_created: false,
            verify_email_sent: false,
        };

        if local_site.require_application {
            response.registration_created = true;
            response.jwt = None;
        }

        // logic block about handling email verification/application messaging (hey you applied wait until admin approves)

        //login_response.jwt = inserted_user.get_jwt(context.master_key());

        Ok(response)
    }
}
