use actix_web::web;
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use reqwest_middleware::ClientWithMiddleware;
use sha2::Sha384;
use std::collections::BTreeMap;
use tinyboards_db::{
    database::PgPool,
    models::{
        board::board::Board,
        comment::comment::Comment,
        post::post::Post,
        secret::Secret,
        site::{registration_application::RegistrationApplication, site::Site, email_verification::{EmailVerificationForm, EmailVerification}, site_invite::{SiteInviteForm, SiteInvite}},
        user::user::User,
    },
    traits::Crud, SiteMode,
};
use tinyboards_db_views::structs::{BoardUserBanView, UserView, BoardView};
use tinyboards_utils::{
    error::TinyBoardsError, 
    rate_limit::RateLimitConfig, 
    settings::structs::{RateLimitSettings, Settings},
    email::send_email,
};
use url::Url;
use uuid::Uuid;

use crate::request::purge_image_from_pictrs;

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
pub async fn blocking<F, T>(pool: &PgPool, f: F) -> Result<T, TinyBoardsError>
where
    F: FnOnce(&mut diesel::PgConnection) -> T + Send + 'static,
    T: Send + 'static,
{
    let pool = pool.clone();
    let res = web::block(move || {
        let mut conn = pool.get().unwrap();
        let res = (f)(&mut conn);
        Ok(res) as Result<T, TinyBoardsError>
    })
    .await
    .map_err(|e| {
        TinyBoardsError::from_error_message(e, "Internal Server Error (Blocking Error)")
    })?;

    res
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), TinyBoardsError> {
    if !(10..=60).contains(&pass.len()) {
        Err(TinyBoardsError::from_message("invalid password"))
    } else {
        Ok(())
    }
}

// less typing !!
type UResultOpt = Result<Option<User>, TinyBoardsError>;
type UResult = Result<User, TinyBoardsError>;

// Tries to take the access token from the auth header and get the user. Returns `Err` if it encounters an error (db error or invalid header format), otherwise `Ok(Some<User>)` or `Ok(None)` is returned depending on whether the token is valid. If being logged in is required, `require_user` should be used.
pub async fn load_user_opt(pool: &PgPool, master_key: &Secret, auth: Option<&str>) -> UResultOpt {
    if auth.is_none() {
        return Ok(None);
    };

    // here it is safe to unwrap, because the above check ensures that `auth` isn't None here
    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message("Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`"));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = master_key.jwt.clone();

    blocking(pool, |conn| User::from_jwt(conn, token, master_key)).await?
}

/**
 A newtype-ish wrapper around UResult with additional methods to make adding additional requirements (enforce lack of site ban, etc) easier.
 Call `unwrap` to get a regular Result.
*/
pub struct UserResult(UResult);

/// Enforces a logged in user. Returns `Ok<User>` if everything is OK, otherwise errors. If being logged in is optional, `load_user_opt` should be used.
pub async fn require_user(pool: &PgPool, master_key: &Secret, auth: Option<&str>) -> UserResult {
    if auth.is_none() {
        let err: UResult = Err(TinyBoardsError::from_message(
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
                    if u.deleted {
                        Err(TinyBoardsError::from_message(
                            "you need to be logged in to do this",
                        ))
                    } else {
                        Ok(u)
                    }
                }
                None => Err(TinyBoardsError::from_message(
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
                if u.deleted {
                    Err(TinyBoardsError::from_message(
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
                    println!("user is banned!");
                    Err(TinyBoardsError::from_message(
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
                if u.admin {
                    Ok(u)
                } else {
                    Err(TinyBoardsError::from_message(
                        "you need to be an admin to do this",
                    ))
                }
            }
            Err(e) => Err(e),
        })
    }

    pub async fn not_banned_from_board(self, board_id: i32, pool: &PgPool) -> Self {
        match self.0 {
            Ok(u) => {
                // skip this check for admins :))))
                if u.admin {
                    return Self(Ok(u));
                }

                let is_banned = blocking(pool, move |conn| {
                    BoardUserBanView::get(conn, u.id, board_id)
                })
                .await;

                let inner = match is_banned {
                    Ok(is_banned) => {
                        if let Ok(board_ban) = is_banned {
                            Err(TinyBoardsError::from_message(
                                format!("You are banned from {}", board_ban.board.name).as_str(),
                            ))
                        } else {
                            Ok(u)
                        }
                    }
                    Err(e) => Err(e),
                };

                Self(inner)
            }
            Err(e) => Self(Err(e)),
        }
    }

    pub async fn require_board_mod(self, board_id: i32, pool: &PgPool) -> Self {
        match self.0 {
            Ok(u) => {
                // admins can do everything
                if u.admin {
                    return Self(Ok(u));
                }

                let is_mod =
                    blocking(pool, move |conn| Board::board_has_mod(conn, board_id, u.id)).await;

                if is_mod.is_err() {
                    return Self(Err(TinyBoardsError::from_message("nerd")));
                };

                let is_mod = is_mod.unwrap();

                let inner = match is_mod {
                    Ok(is_mod) => {
                        if is_mod {
                            Ok(u)
                        } else {
                            Err(TinyBoardsError::from_message(
                                "You must be a mod to do that!",
                            ))
                        }
                    }
                    Err(e) => Err(TinyBoardsError::from_error_message(e, "")),
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
    pool: &PgPool,
    master_key: &Secret,
) -> Result<Option<UserView>, TinyBoardsError> {
    if auth.is_none() {
        return Ok(None);
    }

    // safe unwrap: previous block returns if None
    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message("Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`"));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = master_key.jwt.clone();

    blocking(pool, move |conn| {
        UserView::from_jwt(conn, token, master_key)
    })
    .await?
}

#[tracing::instrument(skip_all)]
pub async fn get_user_view_from_jwt(
    auth: Option<&str>,
    pool: &PgPool,
    master_key: &Secret,
) -> Result<UserView, TinyBoardsError> {
    if auth.is_none() {
        return Err(TinyBoardsError::from_message(
            "you need to be logged in to do that",
        ));
    }

    let user_view = get_user_view_from_jwt_opt(auth, pool, master_key).await?;
    match user_view {
        Some(user_view) => Ok(user_view),
        None => Err(TinyBoardsError::from_message(
            "you need to be logged in to do that",
        )),
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_registration_application(
    site: &Site,
    user_view: &UserView,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    if site.require_application && !user_view.user.admin && !user_view.user.application_accepted {
        let user_id = user_view.user.id;
        let registration = blocking(pool, move |conn| {
            RegistrationApplication::find_by_user_id(conn, user_id)
                .map_err(|_e| TinyBoardsError::from_message("could not find user registration"))
        })
        .await??;

        if let Some(deny_reason) = registration.deny_reason {
            let registration_denied_message = &deny_reason;
            return Err(TinyBoardsError::from_message(registration_denied_message));
        } else {
            return Err(TinyBoardsError::from_message(
                "registration application pending",
            ));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_downvotes_enabled(score: i16, pool: &PgPool) -> Result<(), TinyBoardsError> {
    if score == -1 {
        let site = blocking(pool, move |conn| {
            Site::read_local(conn)
                .map_err(|_e| TinyBoardsError::from_message("could not read site"))
        })
        .await??;

        if !site.enable_downvotes {
            return Err(TinyBoardsError::from_message("downvotes are disabled"));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_private_instance(
    user: &Option<User>,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    if user.is_none() {
        let site = blocking(pool, move |conn| Site::read_local(conn)).await?;

        if let Ok(site) = site {
            if site.private_instance {
                return Err(TinyBoardsError::from_message("instance is private"));
            }
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_board_deleted_or_removed(
    board_id: i32,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    let board = blocking(pool, move |conn| Board::read(conn, board_id))
        .await?
        .map_err(|_e| TinyBoardsError::from_message("couldn't find board"))?;

    if board.deleted || board.removed {
        Err(TinyBoardsError::from_message("board deleted or removed"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_or_removed(
    post_id: i32,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    let post = blocking(pool, move |conn| Post::read(conn, post_id))
        .await?
        .map_err(|_e| TinyBoardsError::from_message("couldn't find post"))?;

    if post.deleted || post.removed {
        Err(TinyBoardsError::from_message("post deleted or removed"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_post_deleted_removed_or_locked(
    post_id: i32,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    let post = blocking(pool, move |conn| Post::read(conn, post_id))
        .await?
        .map_err(|_e| TinyBoardsError::from_message("couldn't find post"))?;

    if post.locked {
        Err(TinyBoardsError::from_message("post locked"))
    } else if post.deleted || post.removed {
        Err(TinyBoardsError::from_message("post deleted or removed"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_comment_deleted_or_removed(
    comment_id: i32,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    let comment = blocking(pool, move |conn| Comment::read(conn, comment_id))
        .await?
        .map_err(|_e| TinyBoardsError::from_message("couldn't find comment"))?;

    if comment.deleted || comment.removed {
        Err(TinyBoardsError::from_message("comment deleted or removed"))
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
    pool: &PgPool,
    user_id: i32,
    board_id: i32,
) -> Result<(), TinyBoardsError> {
    let is_mod_or_admin = blocking(pool, move |conn| {
        BoardView::is_mod_or_admin(conn, user_id, board_id)
    })
    .await?;
    if !is_mod_or_admin {
        return Err(TinyBoardsError::from_message("not a mod or admin"));
    }
    Ok(())
}

pub async fn purge_image_posts_for_user(
    banned_user_id: i32,
    pool: &PgPool,
    settings: &Settings,
    client: &ClientWithMiddleware,
  ) -> Result<(), TinyBoardsError> {
    let posts 
        = blocking(pool, move |conn| { 
            Post::fetch_image_posts_for_creator(conn, banned_user_id)
        })
        .await??;

    for post in posts {
      if let Some(url) = post.url {
        purge_image_from_pictrs(client, settings, &Url::parse(url.as_str()).unwrap()).await.ok();
      }
      if let Some(thumbnail_url) = post.thumbnail_url {
        purge_image_from_pictrs(client, settings, &Url::parse(thumbnail_url.as_str()).unwrap()).await.ok();
      }
    }
  
    blocking(pool, move |conn| {
        Post::remove_post_images_and_thumbnails_for_creator(conn, banned_user_id)
    })
    .await??;
  
    Ok(())
  }

  pub async fn purge_image_posts_for_board(
    banned_board_id: i32,
    pool: &PgPool,
    settings: &Settings,
    client: &ClientWithMiddleware,
  ) -> Result<(), TinyBoardsError> {
    let posts 
        = blocking(pool, move |conn| {
            Post::fetch_image_posts_for_board(conn, banned_board_id)
        })
        .await??;
        
    for post in posts {
      if let Some(url) = post.url {
        purge_image_from_pictrs(client, settings, &Url::parse(url.as_str()).unwrap()).await.ok();
      }
      if let Some(thumbnail_url) = post.thumbnail_url {
        purge_image_from_pictrs(client, settings, &Url::parse(thumbnail_url.as_str()).unwrap()).await.ok();
      }
    }
  
    blocking(pool, move |conn| {
        Post::remove_post_images_and_thumbnails_for_board(conn, banned_board_id)
    })
    .await??;
  
    Ok(())
  }

  /// Send a site invite email
  pub async fn send_invite_email(
    invited_email: &str,
    pool: &PgPool,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {
    
    let form = SiteInviteForm {
        email: invited_email.to_string(),
        verification_code: Uuid::new_v4().to_string(),
    };

    let invite_link = format!(
        "{}/accept_invite/{}",
        settings.get_protocol_and_hostname(),
        &form.verification_code
    );

    // add record for pending site invite to the database
    blocking(pool, move |conn| {
        SiteInvite::create(conn, &form)
    })
    .await??;

    let subject = format!("Invitation to join {}", &settings.hostname);
    let body = format!(
        "You have been invited to make an account on {}!\n\nTo register and make an account please click the link below:\n\n{}",
        &settings.hostname,
        &invite_link
    );

    // send the email
    send_email(&subject, &invited_email, &"", &body, settings)?;

    Ok(())
  }


  /// Send a verification email
  pub async fn send_verification_email(
    user: &User,
    new_email: &str,
    pool: &PgPool,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {

    let form = EmailVerificationForm {
        user_id: user.id,
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
    blocking(pool, move |conn| {
        EmailVerification::create(conn, &form)
    })
    .await??;

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