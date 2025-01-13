
use hmac::{Hmac, Mac};
use anyhow::Context;
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use sha2::Sha384;
use futures::try_join;
use url::{Url, ParseError};
use std::{collections::BTreeMap, fs};
use tinyboards_db::{
    models::{
        board::{boards::{Board, BoardForm}, board_mods::{BoardModerator, ModPerms}},
        comment::comments::Comment,
        post::posts::Post,
        person::user::User,
        secret::Secret,
        site::{registration_applications::RegistrationApplication, email_verification::{EmailVerificationForm, EmailVerification}, uploads::Upload, local_site_rate_limit::LocalSiteRateLimit},
        person::{local_user::*, person_blocks::*, person::{Person, PersonForm}}, site::local_site::LocalSite, apub::instance::Instance, message::message::{MessageForm, Message, MessageNotifForm, MessageNotif},
    },
    traits::Crud, SiteMode, 
    utils::{DbPool, naive_now},
    newtypes::DbUrl,
};
use tinyboards_db_views::{structs::{BoardPersonBanView, BoardView, LocalUserSettingsView, LocalUserView, BoardModeratorView, PersonView}, CommentQuery};
use tinyboards_utils::{
    error::TinyBoardsError, 
    rate_limit::RateLimitConfig, 
    settings::structs::{RateLimitSettings, Settings},
    email::{send_email, /*translations::Lang*/}, location_info, parser::parse_markdown_opt,
};
use uuid::Uuid;
use base64::{
    Engine as _,
    engine::general_purpose,
};

use crate::site::FederatedInstances;

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
type UResultOpt = Result<Option<LocalUserView>, TinyBoardsError>;
type UResult = Result<LocalUserView, TinyBoardsError>;

pub async fn get_user_from_header_opt(pool: &DbPool, master_key: &Secret, auth: Option<&str>) -> Result<Option<User>, TinyBoardsError> {
    if auth.is_none() {
        return Ok(None);
    };

    // here it is safe to unwrap, because the above check ensures that `auth` isn't None here
    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::from_message(400, "Invalid `Authorization` header! It should be `Authorization: Bearer <access token>`"));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    let token = String::from(&auth[7..]);
    let master_key = master_key.jwt.clone();

    let user = User::from_jwt(pool, token, master_key).await?;

    Ok(Some(user))
}

// To be deprecated and removed
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
    let token = String::from(&auth[7..]);
    let master_key = master_key.jwt.clone();

    let local_user = LocalUser::from_jwt(pool, token, master_key).await?.unwrap();
    let view = LocalUserView::read(pool, local_user.id).await?;

    Ok(Some(view))

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

/*pub async fn require_user_opt(pool: &DbPool, master_key: &Secret, auth: Option<&str>) -> Result<Option<LocalUserView>, TinyBoardsError> {
    require_user_opt(auth, pool, master_key).await
}*/

impl From<UResultOpt> for UserResult {
    fn from(r: UResultOpt) -> Self {
        let u_res = match r {
            Ok(u) => match u {
                Some(u) => {
                    if u.local_user.is_deleted {
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
                if u.local_user.is_deleted {
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
                return Self(if u.person.is_banned {
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

    pub fn require_admin(self, permission: AdminPerms) -> Self {
        Self(match self.0 {
            Ok(u) => {
                if u.local_user.has_permission(permission) {
                    Ok(u)
                } else {
                    Err(TinyBoardsError::from_message(
                        403,
                        "insufficient permissions",
                    ))
                }
            }
            Err(e) => Err(e),
        })
    }

    pub async fn not_banned_from_board(self, board_id: i32, pool: &DbPool) -> Self {
        match self.0 {
            Ok(u) => {
                // skip this check for admins :)))) (if they have either the Content or the Boards permission, or both)
                if u.local_user.has_permissions_any(AdminPerms::Content + AdminPerms::Boards) {
                    return Self(Ok(u));
                }
                
                let is_banned = BoardPersonBanView::get(pool, u.person.id, board_id)
                    .await
                    .is_ok();

                let inner = match is_banned {
                    true => Err(TinyBoardsError::from_message(403, "you are banned from this board.")),
                    false => Ok(u),
                };

                Self(inner)
            }
            Err(e) => Self(Err(e)),
        }
    }

    pub async fn require_board_mod(self, pool: &DbPool, board_id: i32, with_permission: ModPerms, rank_required: Option<i32>) -> Self {
        match self.0 {
            Ok(u) => {
                // admins can do everything (in this case, only ones with the Content or Boards permission)
                if u.local_user.has_permissions_any(AdminPerms::Content + AdminPerms::Boards) {
                    return Self(Ok(u));
                }

                let mod_data = Board::board_get_mod(pool, board_id, u.person.id).await;

                let inner = match mod_data {
                    Ok(mod_data) => {
                        match mod_data {
                            Some(mod_data) => {
                                if mod_data.has_permission(with_permission) {
                                    match rank_required {
                                        Some(rank_required) => {
                                            if mod_data.rank > rank_required {
                                                Err(TinyBoardsError::from_message(403, "You are not high enough on the mod hierarchy to do that."))
                                            } else {
                                                Ok(u)
                                            }
                                        }
                                        None => Ok(u)
                                    }
                                } else {
                                    Err(TinyBoardsError::from_message(403, "Missing moderator permissions."))
                                }
                            }
                            None => Err(TinyBoardsError::from_message(403, "You must be a mod to do that!"))
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
pub async fn check_registration_application(
    site: &LocalSite,
    local_user_view: &LocalUserView,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    if site.require_application && local_user_view.local_user.admin_level == 0 && !local_user_view.local_user.is_application_accepted {
        let person_id = local_user_view.local_user.person_id;
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
        let site = LocalSite::read(pool).await?;

        if !site.enable_downvotes {
            return Err(TinyBoardsError::from_message(403, "downvotes are disabled"));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_private_instance(
    user: &Option<LocalUserView>,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    if user.is_none() {
        let site = LocalSite::read(pool).await;

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

    if board.is_deleted || board.is_removed {
        Err(TinyBoardsError::from_message(404, "board deleted or banned"))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_board_ban(
    person_id: i32,
    board_id: i32,
    pool: &DbPool
) -> Result<(), TinyBoardsError> {
    let is_banned = BoardPersonBanView::get(pool, person_id, board_id)
        .await
        .is_ok();
    if is_banned {
        Err(TinyBoardsError::from_message(400, "banned from board"))
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
pub async fn check_person_block(
    my_id: i32,
    other_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    
    let is_blocked = PersonBlock::read(pool, other_id, my_id).await.is_ok();

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

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin_opt(
  pool: &DbPool,
  local_user_view: Option<&LocalUserView>,
  community_id: Option<i32>,
) -> Result<(), TinyBoardsError> {
  if let Some(local_user_view) = local_user_view {
    if let Some(community_id) = community_id {
      is_mod_or_admin(pool, local_user_view.person.id, community_id).await
    } else {
      is_admin(local_user_view)
    }
  } else {
    Err(TinyBoardsError::from_message(403, "not a mod or admin"))
  }
}

pub async fn is_top_admin(pool: &DbPool, person_id: i32) -> Result<(), TinyBoardsError> {
    let admins = PersonView::admins(pool).await?;
    let top_admin = admins
      .first()
      .ok_or_else(|| TinyBoardsError::from_message(400, "no admins"))?;
  
    if top_admin.person.id != person_id {
      return Err(TinyBoardsError::from_message(400, "not top admin"));
    }
    Ok(())
}
  
pub fn is_admin(local_user_view: &LocalUserView) -> Result<(), TinyBoardsError> {
    if !local_user_view.person.is_admin {
        return Err(TinyBoardsError::from_message(400, "not an admin"));
    }
    Ok(())
}

pub async fn send_system_message(pool: &DbPool, recipient_user_id: Option<i32>, recipient_board_id: Option<i32>, body: String) -> Result<(), TinyBoardsError> {
    let body = body + "\n\n*This message was sent automatically. Contact the admins if you have any questions.*";
    let body_html = parse_markdown_opt(&body);

    let form = MessageForm {
        creator_id: Some(1),
        recipient_user_id: Some(recipient_user_id),
        recipient_board_id: Some(recipient_board_id),
        body: Some(body),
        body_html,
        ..MessageForm::default()
    };

    let message = Message::submit(pool, form).await?;

    if let Some(user_id) = recipient_user_id {
        // create notification
        let form = MessageNotifForm {
            recipient_id: Some(user_id),
            pm_id: Some(message.id),
            ..MessageNotifForm::default()
        };

        MessageNotif::create(pool, &form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to save message notification :("))?;
    }

    Ok(())
}

pub async fn purge_local_image_by_url(
    pool: &DbPool,
    img_url: &DbUrl,
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
    local_user: &LocalUser,
    new_email: &str,
    pool: &DbPool,
    settings: &Settings,
  ) -> Result<(), TinyBoardsError> {

    let form = EmailVerificationForm {
        local_user_id: local_user.id,
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
        &local_user.name, 
        &settings.hostname,
        &verify_link
    );

    // send email
    send_email(&subject, new_email, &local_user.name, &body, settings)?;

    Ok(())
  }


  /// Send a verification success email
  pub fn send_email_verification_success(
    user: &LocalUser,
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

    let admins = LocalUserSettingsView::list_admins_with_email(pool).await?;

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

/// Sends a report email to admins
pub async fn send_new_report_email_to_admins(
    reporter_username: &str,
    reported_username: &str,
    pool: &DbPool,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    let admins = LocalUserView::list_admins_with_email(pool).await?;
    let reports_link = &format!("{}/reports", settings.get_protocol_and_hostname());
    let hostname = &settings.hostname;
    for admin in admins {
        let email = &admin.local_user.email.clone().expect("email");
        let subject = &format!("New report created by {reporter_username} for {reported_username} on {hostname}");
        let body = &format!("Please click the link below to view all reports.<br><br><a href=\"{reports_link}\">View Reports</a>");
        send_email(subject, email, &admin.person.name, body, settings)?;
    }
    Ok(())
}


/// gets current site mode
  pub fn get_current_site_mode(site: &LocalSite, site_mode: &Option<SiteMode>) -> SiteMode {
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
        "data:image/jpg;" => Some("jpg"),
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

  pub enum EndpointType {
    Board,
    Person,
    Post,
    Comment,
  }

  pub fn generate_local_apub_endpoint(
    endpoint_type: EndpointType,
    name: &str,
    domain: &str,
  ) -> Result<DbUrl, ParseError> {
    match endpoint_type {
        EndpointType::Board => Ok(Url::parse(&format!("{domain}/+{name}"))?.into()),
        EndpointType::Comment => Ok(Url::parse(&format!("{domain}/comment/{name}"))?.into()),
        EndpointType::Post => Ok(Url::parse(&format!("{domain}/post/{name}"))?.into()),
        EndpointType::Person => Ok(Url::parse(&format!("{domain}/@{name}"))?.into()),
    }
  }

  pub fn generate_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/inbox"))?.into())
  }

  pub fn generate_outbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/outbox"))?.into())
  }

  pub fn generate_subscribers_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/subscribers"))?.into())
  }

  pub fn generate_moderators_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/mods"))?.into())
  }

  pub fn generate_featured_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    Ok(Url::parse(&format!("{actor_id}/featured"))?.into())
  }

  pub fn generate_site_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
    let actor_id: Url = actor_id.clone().into();
    actor_id.clone().set_path("site_inbox");
    Ok(actor_id.into())
  }

  pub fn generate_shared_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, TinyBoardsError> {
    let actor_id: Url = actor_id.clone().into();
    let url = format!(
        "{}://{}{}/inbox",
        &actor_id.clone().scheme(),
        &actor_id.clone().host_str().context(location_info!())?,
        if let Some(port) = actor_id.clone().port() {
            format!(":{}", port)
        } else {
            String::new()
        },
    );
    Ok(Url::parse(&url)?.into())
  }

#[tracing::instrument(skip_all)]
pub async fn build_federated_instances(
  local_site: &LocalSite,
  pool: &DbPool,
) -> Result<Option<FederatedInstances>, TinyBoardsError> {
  if local_site.federation_enabled {
    // TODO I hate that this requires 3 queries
    let (linked, allowed, blocked) = try_join!(
      Instance::linked(pool),
      Instance::allow_list(pool),
      Instance::block_list(pool)
    )?;

    Ok(Some(FederatedInstances {
      linked,
      allowed,
      blocked,
    }))
  } else {
    Ok(None)
  }
}

pub async fn remove_user_data(
        banned_person_id: i32,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError> {
    let person = Person::read(pool, banned_person_id).await?;
    if let Some(avatar) = person.avatar {
        purge_local_image_by_url(pool, &avatar)
            .await
            .ok();
    }
    if let Some(banner) = person.banner {
        purge_local_image_by_url(pool, &banner)
            .await
            .ok();
    }
    if let Some(signature) = person.signature {
        purge_local_image_by_url(pool, &signature)
            .await
            .ok();
    }

    let remove_form = PersonForm {
        avatar: None,
        banner: None,
        signature: None,
        updated: Some(naive_now()),
        ..PersonForm::default()
    };

    Person::update(pool, banned_person_id, &remove_form).await?;

    // Posts
    Post::update_removed_for_creator(pool, banned_person_id, None, true).await?;

    purge_local_image_posts_for_user(banned_person_id, pool).await?;

    let first_mod_boards = BoardModeratorView::get_board_first_mods(pool).await?;

    let banned_user_first_boards: Vec<BoardModeratorView> = first_mod_boards
        .into_iter()
        .filter(|fmb| fmb.moderator.id == banned_person_id)
        .collect();

    for fmb in banned_user_first_boards {
        let board_id = fmb.board.id;
        let form = BoardForm {
            is_removed: Some(true),
            updated: Some(Some(naive_now())),
            ..BoardForm::default()
        };
        let board = Board::update(pool, board_id, &form).await?;

        if let Some(icon) = board.icon {
            purge_local_image_by_url(pool, &icon).await.ok();
        }

        if let Some(banner) = board.banner {
            purge_local_image_by_url(pool, &banner).await.ok();
        }

        let form = BoardForm {
            icon: None,
            banner: None,
            ..BoardForm::default()
        };
        Board::update(pool, board_id, &form).await?;
    }

    Comment::update_removed_for_creator(pool, banned_person_id, true).await?;

    Ok(())
}

pub async fn remove_user_data_in_board(
    board_id: i32,
    banned_person_id: i32,
    pool: &DbPool,
) -> Result<(), TinyBoardsError> {
    Post::update_removed_for_creator(pool, banned_person_id, Some(board_id), true).await?;

    let response = CommentQuery::builder()
        .pool(pool)
        .creator_id(Some(banned_person_id))
        .board_id(Some(board_id))
        .limit(Some(i64::MAX))
        .build()
        .list()
        .await?;

    for comment_view in response.comments {
        let comment_id = comment_view.comment.id;
        Comment::update_removed(pool, comment_id, true).await?;
    }

    Ok(())
}

// pub fn get_interface_language(user: &LocalUserView) -> Lang {
//     lang_str_to_lang(&user.local_user.interface_language)
// }

// fn lang_str_to_lang(lang: &str) -> Lang {
//     let lang_id = LanguageId::new(lang);
//     Lang::from_language_id(&lang_id).unwrap_or_else(|| {
//       let en = LanguageId::new("en");
//       Lang::from_language_id(&en).expect("default language")
//     })
// }

pub async fn delete_user_account(
    person_id: i32,
    pool: &DbPool,
  ) -> Result<(), TinyBoardsError> {
    // Delete their images
    let person = Person::read(pool, person_id).await?;
    if let Some(avatar) = person.avatar {
      purge_local_image_by_url(pool, &avatar).await?;
    }
    if let Some(banner) = person.banner {
      purge_local_image_by_url(pool, &banner).await?;
    }
    // No need to update avatar and banner, those are handled in Person::delete_account
  
    // Comments
    Comment::permadelete_for_creator(pool, person_id)
      .await
      .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't update comment"))?;
  
    // Posts
    Post::permadelete_for_creator(pool, person_id)
      .await
      .map_err(|e| TinyBoardsError::from_error_message(e, 500, "couldn't update post"))?;
  
    // Purge image posts
    purge_local_image_posts_for_user(person_id, pool).await?;
  
    // Leave boards they mod
    BoardModerator::leave_all_boards(pool, person_id).await?;
  
    Person::delete_account(pool, person_id).await?;
  
    Ok(())
  }

  pub fn check_private_instance_and_federation_enabled(
    local_site: &LocalSite,
  ) -> Result<(), TinyBoardsError> {
    if local_site.private_instance && local_site.federation_enabled {
        return Err(TinyBoardsError::from_message(400, "cannot have private instance and federation enabled at the same time."));
    }
    Ok(())
  }

  pub fn local_site_rate_limit_to_rate_limit_config(
    local_site_rate_limit: &LocalSiteRateLimit,
  ) -> RateLimitConfig {
    let l = local_site_rate_limit;
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
