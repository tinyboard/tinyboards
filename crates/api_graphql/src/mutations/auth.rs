use crate::helpers::apub::{
    generate_inbox_url, generate_local_apub_endpoint, generate_shared_inbox_url, EndpointType,
};
/**
 * Login and registration
 **/
use crate::{DbPool, LoggedInUser, MasterKey, Settings};
use async_graphql::*;
use regex::Regex;
use tinyboards_db::models::person::local_user::LocalUser as DbLocalUser;
use tinyboards_db::models::person::local_user::*;
use tinyboards_db::models::person::person::*;
use tinyboards_db::models::site::registration_applications::RegistrationApplication;
use tinyboards_db::models::site::registration_applications::RegistrationApplicationForm;
use tinyboards_db::models::site::site_invite::SiteInvite as DbSiteInvite;
use tinyboards_db::models::site::{local_site::LocalSite as DbLocalSite, site::Site as DbSite};
use tinyboards_db::traits::Crud;
//use tinyboards_federation::http_signatures::generate_actor_keypair;
use tinyboards_utils::passhash::{hash_password, verify_password};
use tinyboards_utils::TinyBoardsError;
use url::Url;

#[derive(Default)]
pub struct Auth;

#[derive(SimpleObject)]
pub struct LoginResponse {
    token: String,
}

#[derive(SimpleObject)]
pub struct SignupResponse {
    token: Option<String>,
    account_created: bool,
    application_submitted: bool,
}

#[Object]
impl Auth {
    pub async fn login(
        &self,
        ctx: &Context<'_>,
        username_or_email: String,
        password: String,
    ) -> Result<LoginResponse> {
        let v = ctx.data_unchecked::<LoggedInUser>().inner();

        if v.is_some() {
            return Err(TinyBoardsError::from_message(400, "You are already logged in").into());
        }

        let pool = ctx.data::<DbPool>()?;
        let master_key = ctx.data::<MasterKey>()?;

        // attempted login with email - look up account by email
        let u = if username_or_email.contains('@') {
            DbLocalUser::get_by_email(pool, &username_or_email).await
        } else {
            // look up account by username
            DbLocalUser::get_by_name(pool, &username_or_email).await
        }?;

        // password check - also deleted accounts cannot be logged into
        if !verify_password(&u.passhash, &password) || u.is_deleted {
            return Err(TinyBoardsError::from_message(
                401,
                "Username, email address or password invalid.",
            )
            .into());
        }

        let site = DbLocalSite::read(pool).await?;

        // if application mode is enabled, each acccount must be admin approved before it can be used
        if site.require_application && !u.is_application_accepted {
            return Err(TinyBoardsError::from_message(
                403,
                "You cannot use your account yet because your application hasn't been accepted.",
            )
            .into());
        }

        // all good: generate access token
        Ok(LoginResponse {
            token: u.get_jwt(master_key.as_ref()),
        })
    }

    pub async fn register(
        &self,
        ctx: &Context<'_>,
        username: String,
        display_name: Option<String>,
        email: Option<String>,
        password: String,
        invite_code: Option<String>,
        application_answer: Option<String>,
    ) -> Result<SignupResponse> {
        let v = ctx.data_unchecked::<LoggedInUser>().inner();

        if v.is_some() {
            return Err(TinyBoardsError::from_message(400, "You are already logged in").into());
        }

        let pool = ctx.data::<DbPool>()?;
        let master_key = ctx.data::<MasterKey>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        let site = DbLocalSite::read(pool).await?;
        let instance_id = DbSite::read_local(pool).await?.instance_id;

        let protocol_and_hostname = settings.get_protocol_and_hostname();

        if site.invite_only && invite_code.is_none() {
            return Err(
                TinyBoardsError::from_message(403, "You need an invite to register.").into(),
            );
        }

        if site.require_application && application_answer.is_none() {
            return Err(
                TinyBoardsError::from_message(400, "You need to write an application.").into(),
            );
        }

        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{0,29}$").unwrap();
        if !re.is_match(&username) {
            return Err(TinyBoardsError::from_message(400, "Invalid username.").into());
        }

        // PASSWORD CHECK
        // password_length_check(&data.password)?;
        if !(8..60).contains(&password.len()) {
            return Err(TinyBoardsError::from_message(
                400,
                "Your password must be between 8 and 60 characters long.",
            )
            .into());
        }

        if site.require_email_verification && email.is_none() {
            return Err(TinyBoardsError::from_message(
                400,
                "Email verification is required, please provide an email.",
            )
            .into());
        }

        let mut avatar_url =
            Url::parse(format!("{}/media/default_pfp.png", &protocol_and_hostname).as_str())?;

        let invite = if site.invite_only {
            let invite = DbSiteInvite::read_for_token(pool, &invite_code.unwrap())
                .await
                .map_err(|e| TinyBoardsError::from_error_message(e, 403, "Invalid invite"))?;
            Some(invite)
        } else {
            None
        };

        //let actor_keypair = generate_actor_keypair()?;

        let actor_id =
            generate_local_apub_endpoint(EndpointType::Person, &username, &protocol_and_hostname)?;

        // if we have a default avatar for the site, then use it
        if site.default_avatar.is_some() {
            avatar_url = Url::parse(&site.default_avatar.unwrap().clone())?;
        }

        // now we need to create both a local_user and a person (for apub)
        let person_form = PersonForm {
            name: Some(username.clone()),
            display_name: Some(display_name.unwrap_or(username.clone())),
            actor_id: Some(actor_id.clone()),
            private_key: Some("-".to_string()),
            public_key: Some("-".to_string()),
            inbox_url: Some(generate_inbox_url(&actor_id)?),
            shared_inbox_url: Some(generate_shared_inbox_url(&actor_id)?),
            instance_id: Some(instance_id),
            avatar: Some(avatar_url.into()),
            ..PersonForm::default()
        };

        let person = Person::create(pool, &person_form).await?;

        let passhash = hash_password(password);

        let local_user_form = LocalUserForm {
            name: Some(username),
            email: Some(email),
            passhash: Some(passhash),
            person_id: Some(person.id),
            ..LocalUserForm::default()
        };

        let inserted_local_user = match LocalUser::create(pool, &local_user_form).await {
            Ok(lu) => lu,
            Err(e) => {
                let err_type = if e.to_string()
                    == "duplicate key value violates unique constraint \"local_user_email_key\""
                {
                    "email address"
                } else {
                    "username"
                };

                // if local_user creation failed then delete the person
                Person::delete(pool, person.id).await?;

                return Err(TinyBoardsError::from_error_message(
                    e,
                    500,
                    &format!("A user with that {} already exists.", err_type),
                )
                .into());
            }
        };

        //let inserted_user = LocalUser::register(context.pool(), user_form).await?;

        // if the user was invited, invalidate the invite token here by removing from db
        if site.invite_only {
            DbSiteInvite::delete(pool, invite.unwrap().id).await?;
        }

        // if site is in application mode, add the application to the database
        if site.require_application {
            let form = RegistrationApplicationForm {
                person_id: person.id,
                answer: application_answer,
                ..RegistrationApplicationForm::default()
            };

            RegistrationApplication::create(pool, &form).await?;
        }

        // TODO: send notification to admins about registration application

        // TODO: email verification, if that's required

        Ok(SignupResponse {
            token: {
                if site.open_registration || site.invite_only {
                    Some(inserted_local_user.get_jwt(master_key.as_ref()))
                } else {
                    None
                }
            },
            account_created: !site.require_application,
            application_submitted: site.require_application,
        })
    }
}
