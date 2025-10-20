/**
 * Login and registration
 **/
use crate::{utils::auth::{password_length_check, get_jwt}, DbPool, LoggedInUser, MasterKey, Settings};
use async_graphql::*;
use regex::Regex;

use tinyboards_db::models::user::user::*;

use tinyboards_db::models::site::registration_applications::RegistrationApplication;
use tinyboards_db::models::site::registration_applications::RegistrationApplicationForm;
use tinyboards_db::models::site::site_invite::SiteInvite as DbSiteInvite;
use tinyboards_db::models::site::site::Site as DbSite;
use tinyboards_db::models::secret::Secret;
use diesel_async::RunQueryDsl;
use tinyboards_db::RegistrationMode;
use tinyboards_db::traits::Crud;
//use tinyboards_federation::http_signatures::generate_actor_keypair;
use tinyboards_utils::passhash::{hash_password, verify_password};
use tinyboards_utils::content_filter::ContentFilter;
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
        let _master_key = ctx.data::<MasterKey>()?;

        // attempted login with email - look up account by email
        let u = if username_or_email.contains('@') {
            User::get_by_email(pool, &username_or_email).await
        } else {
            // look up account by username
            User::get_by_name(pool, username_or_email.to_string()).await
        }?;

        // password check - also deleted accounts cannot be logged into
        if !verify_password(&u.passhash, &password) || u.is_deleted {
            return Err(TinyBoardsError::from_message(
                401,
                "Username, email address or password invalid.",
            )
            .into());
        }

        let site = DbSite::read(pool).await?;

        // Check if user has a pending application
        // Only block login if they actually submitted an application that's pending
        if !u.is_application_accepted {
            use tinyboards_db::models::site::registration_applications::RegistrationApplication;

            // Check if there's a pending application for this user
            let has_pending_application = RegistrationApplication::find_by_user_id(pool, u.id)
                .await
                .is_ok();

            if has_pending_application {
                return Err(TinyBoardsError::from_message(
                    403,
                    "You cannot use your account yet because your application hasn't been accepted.",
                )
                .into());
            }
        }

        // all good: generate access token
        // Read the secret from database to generate proper JWT
        use tinyboards_db::schema::secret::dsl::secret as secret_table;
        let mut conn = pool.get().await?;
        let jwt_secret = secret_table.first::<Secret>(&mut conn).await.map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Failed to load JWT secret")
        })?;

        Ok(LoginResponse {
            token: get_jwt(u.id, &u.name, &jwt_secret)?,
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
        let _master_key = ctx.data::<MasterKey>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        let site = DbSite::read(pool).await?;

        let protocol_and_hostname = settings.get_protocol_and_hostname();

        let registration_mode = site.get_registration_mode();

        // Check registration policy constraints
        match registration_mode {
            RegistrationMode::Closed => {
                return Err(TinyBoardsError::from_message(403, "Registration is closed.").into());
            }
            RegistrationMode::InviteOnlyAdmin | RegistrationMode::InviteOnlyUser => {
                if invite_code.is_none() {
                    return Err(TinyBoardsError::from_message(403, "You need an invite to register.").into());
                }
            }
            RegistrationMode::RequireApplication => {
                if application_answer.is_none() {
                    return Err(TinyBoardsError::from_message(400, "You need to write an application.").into());
                }
            }
            RegistrationMode::OpenWithEmailVerification => {
                if email.is_none() {
                    return Err(TinyBoardsError::from_message(400, "Email verification is required, please provide an email.").into());
                }
            }
            RegistrationMode::Open => {
                // No additional restrictions
            }
        }

        let re = Regex::new(r"^[A-Za-z][A-Za-z0-9_]{0,29}$").unwrap();
        if !re.is_match(&username) {
            return Err(TinyBoardsError::from_message(400, "Invalid username.").into());
        }

        // Validate username against content filters
        ContentFilter::validate_username(
            &site.word_filter_enabled,
            &site.word_filter_applies_to_usernames,
            &site.filtered_words,
            &username,
        )?;

        // PASSWORD CHECK
        password_length_check(&password)?;

        let invite = match registration_mode {
            RegistrationMode::InviteOnlyAdmin | RegistrationMode::InviteOnlyUser => {
                let invite = DbSiteInvite::read_for_token(pool, &invite_code.unwrap())
                    .await
                    .map_err(|e| TinyBoardsError::from_error_message(e, 403, "Invalid invite"))?;
                Some(invite)
            }
            _ => None,
        };

        //let actor_keypair = generate_actor_keypair()?;

        // Use site's default avatar if configured, otherwise NULL to let frontend handle fallback
        let avatar_url = site.default_avatar.as_ref().and_then(|url| Url::parse(url).ok());

        // create user account
        let passhash = hash_password(password);
        let requires_application = registration_mode == RegistrationMode::RequireApplication;

        let user_form = UserForm {
            name: Some(username.clone()),
            display_name: Some(display_name.unwrap_or(username.clone())),
            email: Some(email),
            passhash: Some(passhash),
            avatar: Some(avatar_url.map(|u| u.into())),
            is_application_accepted: Some(!requires_application),
            ..UserForm::default()
        };

        let inserted_user = match User::create(pool, &user_form).await {
            Ok(u) => u,
            Err(e) => {
                let err_type = if e.to_string()
                    == "duplicate key value violates unique constraint \"users_email_key\""
                {
                    "email address"
                } else {
                    "username"
                };

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
        if let Some(invite) = invite {
            DbSiteInvite::delete(pool, invite.id).await?;
        }

        // if site is in application mode, add the application to the database
        if registration_mode == RegistrationMode::RequireApplication {
            let form = RegistrationApplicationForm {
                user_id: inserted_user.id,
                answer: application_answer,
                ..RegistrationApplicationForm::default()
            };

            RegistrationApplication::create(pool, &form).await?;
        }

        // Send notification to admins about new registration - commented out for now
        // send_registration_notification_to_admins(pool, &user_view.user.name).await?;

        // TODO: email verification, if that's required

        // Read the secret from database to generate proper JWT
        use tinyboards_db::schema::secret::dsl::secret as secret_table;
        let mut conn = pool.get().await?;
        let jwt_secret = secret_table.first::<Secret>(&mut conn).await.map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Failed to load JWT secret")
        })?;

        Ok(SignupResponse {
            token: {
                match registration_mode {
                    RegistrationMode::Open
                    | RegistrationMode::OpenWithEmailVerification
                    | RegistrationMode::InviteOnlyAdmin
                    | RegistrationMode::InviteOnlyUser => {
                        Some(get_jwt(inserted_user.id, &inserted_user.name, &jwt_secret)?)
                    }
                    RegistrationMode::RequireApplication
                    | RegistrationMode::Closed => None,
                }
            },
            account_created: registration_mode != RegistrationMode::RequireApplication,
            application_submitted: registration_mode == RegistrationMode::RequireApplication,
        })
    }
}
