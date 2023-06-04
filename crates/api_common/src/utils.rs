
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use sha2::Sha384;
use std::{collections::BTreeMap, fs};
use tinyboards_db::{
    models::{
        board::boards::Board,
        comment::comments::Comment,
        post::posts::Post,
        secret::Secret,
        site::{registration_applications::RegistrationApplication, site::Site, email_verification::{EmailVerificationForm, EmailVerification}, uploads::Upload},
        person::{local_user::*, person_blocks::*},
    },
    traits::Crud, SiteMode, 
    utils::DbPool,
};
use tinyboards_db_views::structs::{BoardPersonBanView, BoardView, PersonView, LocalUserSettingsView};
use tinyboards_utils::{
    error::TinyBoardsError, 
    rate_limit::RateLimitConfig, 
    settings::structs::{RateLimitSettings, Settings},
    email::send_email,
};
use uuid::Uuid;
use base64::{
    Engine as _,
    engine::{general_purpose},
};

pub fn get_jwt(uid: i32, uname: &str, master_key: &Secret) -> String {
    let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.jwt.as_bytes()).unwrap();
    let header = Header {
        algorithm: AlgorithmType::Hs384,
        ..Default::default()
    };

    let mut claims = BTreeMap::new();
    claims.insert("uid", uid.to_string());
    claims.insert("uname", uname.to_string());

    let token = Token::new(header, claims)
        .sign_with_key(&key)
        .unwrap()
        .as_str()
        .to_string();

    token
}

/**
 * Use this function to do db operations.
 * Takes a reference to a connection pool as one argument (`TinyBoardsContext::pool(&self)`) and a closure as another.
 * Passes a mutable reference to an open connection to the db to the closure and executes it. Then, the result of the closure is returned.
 * *(shamelessly stolen from lemmy)*
 */
// pub async fn blocking<F, T>(pool: &DbPool, f: F) -> Result<T, TinyBoardsError>
// where
//     F: FnOnce(&mut diesel::PgConnection) -> T + Send + 'static,
//     T: Send + 'static,
// {
//     let pool = pool.clone();
//     let res = web::block(move || {
//         let mut conn = pool.get().unwrap();
//         let res = (f)(&mut conn);
//         Ok(res) as Result<T, TinyBoardsError>
//     })
//     .await
//     .map_err(|e| {
//         TinyBoardsError::from_error_message(e, 500, "Internal Server Error (Blocking Error)")
//     })?;

//     res
// }

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), TinyBoardsError> {
    if !(10..=60).contains(&pass.len()) {
        Err(TinyBoardsError::from_message(400, "password length must be between 10-60 characters"))
    } else {
        Ok(())
    }
}

// less typing !!
type UResultOpt = Result<Option<LocalUser>, TinyBoardsError>;
type UResult = Result<LocalUser, TinyBoardsError>;

// Tries to take the access token from the auth header and get the user. Returns `Err` if it encounters an error (db error or invalid header format), otherwise `Ok(Some<User>)` or `Ok(None)` is returned depending on whether the token is valid. If being logged in is required, `require_user` should be used.
pub async fn load_user_opt(pool: &DbPool, master_key: &Secret, auth: Option<&str>) -> UResultOpt {
    if auth.is_none() {
        return Ok(None);
    };

    // here it is safe to unwrap, because the above check ensures that `auth` isn't None here
    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message(400, "Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`"));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = master_key.jwt.clone();

    LocalUser::from_jwt(pool, token, master_key).await
}

/**
 A newtype-ish wrapper around UResult with additional methods to make adding additional requirements (enforce lack of site ban, etc) easier.
 Call `unwrap` to get a regular Result.
*/
pub struct UserResult(UResult);

/// Enforces a logged in user. Returns `Ok<User>` if everything is OK, otherwise errors. If being logged in is optional, `load_user_opt` should be used.
pub async fn require_user(pool: &DbPool, master_key: &Secret, auth: Option<&str>) -> UserResult {
    if auth.is_none() {
        let err: UResult = Err(TinyBoardsError::from_message(
            401,
            "you need to be logged in to do this",
        ));
        return err.into();
    }

    load_user_opt(pool, master_key, auth).await.into()
}

impl From<UResultOpt> for UserResult {
    fn from(r: UResultOpt) -> Self {
        let u_res = match r {
            Ok(u) => match u {
                Some(u) => {
                    if u.is_deleted {
                        Err(TinyBoardsError::from_message(
                            401,
                            "you need to be logged in to do this",
                        ))
                    } else {
                        Ok(u)
                    }
                }
                None => Err(TinyBoardsError::from_message(
                    401,
                    "you need to be logged in to do this",
                )),
            },
            Err(e) => Err(e),
        };

        Self(u_res)
    }
}

impl From<UResult> for UserResult {
    fn from(r: UResult) -> Self {
        Self(match r {
            Ok(u) => {
                if u.is_deleted {
                    Err(TinyBoardsError::from_message(
                        401,
                        "you need to be logged in to do this",
                    ))
                } else {
                    Ok(u)
                }
            }
            Err(e) => Err(e),
        })
    }
}

impl UserResult {
    pub fn unwrap(self) -> UResult {
        self.0
    }

    pub fn not_banned(self) -> Self {
        match self.0 {
            Ok(u) => {
                return Self(if u.has_active_ban() {
                    Err(TinyBoardsError::from_message(
                        403,
                        "you are banned from the site",
                    ))
                } else {
                    Ok(u)
                });
            }
            Err(e) => Self(Err(e)),
        }
    }

    pub fn require_admin(self) -> Self {
        Self(match self.0 {
            Ok(u) => {
                if u.is_admin {
                    Ok(u)
                } else {
                    Err(TinyBoardsError::from_message(
                        403,
                        "you need to be an admin to do this",
                    ))
                }
            }
            Err(e) => Err(e),
        })
    }

    pub async fn not_banned_from_board(self, board_id: i32, pool: &DbPool) -> Self {
        match self.0 {
            Ok(u) => {
                // skip this check for admins :))))
                if u.is_admin {
                    return Self(Ok(u));
                }
                
                let is_banned = BoardPersonBanView::get(pool, u.id, board_id)
                    .await
                    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "fetching board user ban failed"));

                let inner = match is_banned {
                    Ok(is_banned) => {
                        Err(TinyBoardsError::from_message(403, format!("You are banned from /b/{}", is_banned.board.name).as_str()))
                    }
                    Err(e) => Err(e),
                };

                Self(inner)
            }
            Err(e) => Self(Err(e)),
        }
    }

    pub async fn require_board_mod(self, board_id: i32, pool: &DbPool) -> Self {
        match self.0 {
            Ok(u) => {
                // admins can do everything
                if u.is_admin {
                    return Self(Ok(u));
                }

                let is_mod = Board::board_has_mod(pool, board_id, u.id).await;

                let inner = match is_mod {
                    Ok(is_mod) => {
                        if is_mod {
                            Ok(u)
                        } else {
                            Err(TinyBoardsError::from_message(
                                403,
                                "You must be a mod to do that!",
                            ))
                        }
                    }
                    Err(e) => Err(TinyBoardsError::from(e)),
                };

                Self(inner)
            }
            Err(e) => Self(Err(e)),
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn get_user_view_from_jwt_opt(
    auth: Option<&str>,
    pool: &DbPool,
    master_key: &Secret,
) -> Result<Option<LocalUser>, TinyBoardsError> {
    if auth.is_none() {
        return Ok(None);
    }

    // safe unwrap: previous block returns if None
    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message(400, "Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`"));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = master_key.jwt.clone();


    LocalUser::from_jwt(pool, token, master_key).await
}

#[tracing::instrument(skip_all)]
pub async fn get_user_view_from_jwt(
    auth: Option<&str>,
    pool: &DbPool,
    master_key: &Secret,
) -> Result<LocalUser, TinyBoardsError> {
    if auth.is_none() {
        return Err(TinyBoardsError::from_message(
            401,
            "you need to be logged in to do that",
        ));
    }

    let user_view = get_user_view_from_jwt_opt(auth, pool, master_key).await?;
    match user_view {
        Some(user_view) => Ok(user_view),
        None => Err(TinyBoardsError::from_message(
            401,
            "you need to be logged in to do that",
        )),
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_registration_application(
    site: &Site,
    user_view: &UserView,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    if site.require_application && !user_view.user.is_admin && !user_view.user.is_application_accepted {
        let person_id = user_view.user.id;
        let registration = RegistrationApplication::find_by_person_id(pool, person_id).await?;

        if let Some(deny_reason) = registration.deny_reason {
            let registration_denied_message = &deny_reason;
            return Err(TinyBoardsError::from_message(403, registration_denied_message));
        } else {
            return Err(TinyBoardsError::from_message(
                400,
                "registration application pending",
            ));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_downvotes_enabled(score: i16, pool: &DbPool) -> Result<(), TinyBoardsError> {
    if score == -1 {
        let site = Site::read_local(pool).await?;

        if !site.enable_downvotes {
            return Err(TinyBoardsError::from_message(403, "downvotes are disabled"));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_private_instance(
    user: &Option<User>,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    if user.is_none() {
        let site = Site::read_local(pool).await;

        if let Ok(site) = site {
            if site.private_instance {
                return Err(TinyBoardsError::from_message(403, "instance is private"));
            }
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_board_deleted_or_removed(
    board_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let board = Board::read(pool, board_id).await.map_err(|_e| TinyBoardsError::from_message(404, "couldn't find board"))?;

    if board.is_deleted || board.is_banned {
        Err(TinyBoardsError::from_message(404, "board deleted or banned"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_or_removed(
    post_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let post = Post::read(pool, post_id).await.map_err(|_e| TinyBoardsError::from_message(404, "couldn't find post"))?;

    if post.is_deleted || post.is_removed {
        Err(TinyBoardsError::from_message(404, "post deleted or removed"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_removed_or_locked(
    post_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    let post = Post::read(pool, post_id).await.map_err(|_e| TinyBoardsError::from_message(404, "couldn't find post"))?;

    if post.is_locked {
        Err(TinyBoardsError::from_message(403, "post locked"))
    } else if post.is_deleted || post.is_removed {
        Err(TinyBoardsError::from_message(404, "post deleted or removed"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_user_block(
    my_id: i32,
    other_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    
    let is_blocked = UserBlock::read(pool, other_id, my_id).await.is_ok();

    if is_blocked {
        Err(TinyBoardsError::from_message(405, "user is blocking you"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_comment_deleted_or_removed(
    comment_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {

    let comment = Comment::read(pool, comment_id).await.map_err(|_e| TinyBoardsError::from_message(404, "couldn't find comment"))?;

    if comment.is_deleted || comment.is_removed {
        Err(TinyBoardsError::from_message(404, "comment deleted or removed"))
    } else {
        Ok(())
    }
}

pub fn get_rate_limit_config(rate_limit_settings: &RateLimitSettings) -> RateLimitConfig {
    let l = rate_limit_settings;
    RateLimitConfig {
        message: l.message,
        message_per_second: l.message_per_second,
        post: l.post,
        post_per_second: l.post_per_second,
        register: l.register,
        register_per_second: l.register_per_second,
        image: l.image,
        image_per_second: l.image_per_second,
        comment: l.comment,
        comment_per_second: l.comment_per_second,
        search: l.search,
        search_per_second: l.search_per_second,
    }
}

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin(
    pool: &DbPool,
    person_id: i32,
    board_id: i32,
) -> Result<(), TinyBoardsError> {
    let is_mod_or_admin = BoardView::is_mod_or_admin(pool, person_id, board_id).await;

    if !is_mod_or_admin {
        return Err(TinyBoardsError::from_message(403, "not a mod or admin"));
    }
    Ok(())
}

pub async fn purge_local_image_by_url(
    pool: &DbPool,
    img_url: &str,
) -> Result<(), TinyBoardsError> {

    // get the file by URL
    let file = Upload::find_by_url(pool, img_url).await?;
    // remove file from local disk
    fs::remove_file(file.file_path.clone())?;
    // delete DB entry
    Upload::delete(pool, file.id.clone()).await?;

    Ok(())
}

pub async fn purge_local_image_posts_for_user(
    banned_person_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {

    let posts = Post::fetch_image_posts_for_creator(pool, banned_person_id).await?;

    for post in posts {
        if let Some(url) = post.url {

            purge_local_image_by_url(pool, &url).await.ok();
        }
        if let Some(thumbnail_url) = post.thumbnail_url {
            purge_local_image_by_url(pool, &thumbnail_url).await.ok();
        }
    }

    Post::remove_post_images_and_thumbnails_for_creator(pool, banned_person_id).await?;

    Ok(())
}

pub async fn purge_local_image_posts_for_board(
    banned_board_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {

    let posts = Post::fetch_image_posts_for_board(pool, banned_board_id).await?;

    for post in posts {
        if let Some(url) = post.url {
            purge_local_image_by_url(pool, &url).await.ok();
        }
        if let Some(thumbnail_url) = post.thumbnail_url {
            purge_local_image_by_url(pool, &thumbnail_url).await.ok();
        }
    }

    Post::remove_post_images_and_thumbnails_for_board(pool, banned_board_id).await?;

    Ok(())
  }

  /// Send password reset email
  pub async fn send_password_reset_email(
    username: &str,
    email: &str,
    reset_link: &str,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {

    let subject = format!("Password Reset for your {} Account", &settings.hostname);
    let body = format!(
        "You have requested to reset the password for your {} account. Please visit the link below in order to reset your password.\n\n\n{}",
        settings.get_protocol_and_hostname(),
        &reset_link, 
    );

    // send email
    send_email(&subject, email, username, &body, settings)?;

    Ok(())
  }

  /// Send password reset success email
  pub async fn send_password_reset_success_email(
    username: &str,
    email: &str,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {

    let subject = format!("Successfully Reset Password for your {} Account", &settings.hostname);
    let body = format!(
        "You have successfully reset the password for your {} account.",
        settings.get_protocol_and_hostname()
    );

    // send email
    send_email(&subject, email, username, &body, settings)?;

    Ok(())
  }

  /// Send a verification email
  pub async fn send_verification_email(
    user: &User,
    new_email: &str,
    pool: &DbPool,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {

    let form = EmailVerificationForm {
        person_id: user.id,
        email: new_email.to_string(),
        verification_code: Uuid::new_v4().to_string(),
    };

    // link for verification
    let verify_link = format!(
        "{}/verify_email/{}",
        settings.get_protocol_and_hostname(),
        &form.verification_code
    );

    // add record for pending email verification to the database
    EmailVerification::create(pool, &form).await?;

    let subject = format!("Email Verification for your {} Account", &settings.hostname);
    let body = format!(
        "Thank you {} for registering for an account at {}. Please click the link below in order to verify your email: \n\n {}", 
        &user.name, 
        &settings.hostname,
        &verify_link
    );

    // send email
    send_email(&subject, new_email, &user.name, &body, settings)?;

    Ok(())
  }


  /// Send a verification success email
  pub fn send_email_verification_success(
    user: &User,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {
    let email = &user.email.clone().expect("email");
    let subject = format!("Email Verification Succeeded for your {} Account", &settings.hostname);
    let body = format!("Your email for your new {} account was successful!", &settings.hostname);
    send_email(&subject, &email, &user.name, &body, settings)?;
    Ok(())
}

/// Send a approval email to applicant when admin approves
pub async fn send_application_approval_email(
    applicant_username: &str,
    applicant_email: &str,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    let subject = format!("Welcome To {} - {}!", settings.hostname, applicant_username);
    let body = format!("Your application to {} has been successfully approved by an admin!\n\nYou can now login to your {} account with the credentials you used when you applied.", settings.hostname, settings.hostname);
    send_email(&subject, applicant_email, applicant_username, &body, settings)?;
    Ok(())
}   

/// Sends email to admins after a user applies
  pub async fn send_new_applicant_email_to_admins(
    applicant_username: &str,
    pool: &DbPool,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {

    let admins = UserSettingsView::list_admins_with_email(pool).await?;

    let application_link = &format!(
        "{}/admin/applications",
        settings.get_protocol_and_hostname(),
    );

    for admin in &admins {
        let email = &admin.settings.email.clone().expect("email");
        let subject = format!("New Account Up For Review - {}", applicant_username);
        let body = format!("A new user named {} has registered to {}!\n\nPlease review their application here: {}", applicant_username, settings.hostname, application_link);
        send_email(&subject, email, &admin.settings.name, &body, settings)?;
    }

    Ok(())
}


/// gets current site mode
  pub fn get_current_site_mode(site: &Site, site_mode: &Option<SiteMode>) -> SiteMode {
    let mut current_mode = match site_mode {
        Some(SiteMode::OpenMode) => SiteMode::OpenMode,
        Some(SiteMode::ApplicationMode) => SiteMode::ApplicationMode,
        Some(SiteMode::InviteMode) => SiteMode::InviteMode,
        None => SiteMode::OpenMode, 
    };

    if site_mode.is_none() {
        if site.open_registration {
            current_mode = SiteMode::OpenMode;
        }
        if site.require_application {
            current_mode = SiteMode::ApplicationMode;
        }
        if site.invite_only {
            current_mode = SiteMode::InviteMode;
        }
    }

    current_mode
  }

  pub fn decode_base64_image(encoded_img: String) -> Result<(Vec<u8>, String), TinyBoardsError> {

    let img_split = encoded_img.split("base64,");
    let img_vec = img_split.collect::<Vec<&str>>();

    let img_info = img_vec[0];
    let img_data = img_vec[1];

    let img_fmt_string = match img_info {
        "data:image/png;" => Some("png"),
        "data:image/jpeg;" => Some("jpeg"),
        "data:image/gif;" => Some("gif"),
        "data:image/webp;" => Some("webp"),
        _ => None,
    };

    if img_fmt_string.is_none() {
        return Err(TinyBoardsError::from_message(400, "invalid image format."));
    }

    let bytes = general_purpose::STANDARD
        .decode(img_data)
        .unwrap();

    let file_name = format!("image.{}", img_fmt_string.unwrap());

    Ok((bytes, file_name))
  }