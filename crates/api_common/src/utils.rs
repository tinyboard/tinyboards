use actix_web::web;
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use sha2::Sha384;
use std::collections::BTreeMap;
use tinyboards_db::{
    database::PgPool,
    impls::user::is_banned,
    models::{
        board::board::Board,
        comment::comment::Comment,
        post::post::Post,
        secret::Secret,
        site::{registration_application::RegistrationApplication, site::Site},
        user::user::User,
    },
    traits::Crud,
};
use tinyboards_db_views::structs::{BoardUserBanView, BoardView, UserView};
use tinyboards_utils::error::TinyBoardsError;

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
    .map_err(|_| {
        TinyBoardsError::new(500, String::from("Internal Server Error (BlockingError)"))
    })?;

    res
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), TinyBoardsError> {
    if !(10..=60).contains(&pass.len()) {
        Err(TinyBoardsError {
            message: String::from("invalid password"),
            error_code: 400,
        })
    } else {
        Ok(())
    }
}

// Tries to take the access token from the auth header and get the user. Returns `Err` if it encounters an error (db error or invalid header format), otherwise `Ok(Some<User>)` or `Ok(None)` is returned depending on whether the token is valid. If being logged in is required, `require_user` should be used.
pub async fn load_user_opt(
    pool: &PgPool,
    master_key: &Secret,
    auth: Option<&str>,
) -> Result<Option<User>, TinyBoardsError> {
    if auth.is_none() {
        return Ok(None);
    };

    // here it is safe to unwrap, because the above check ensures that `auth` isn't None here
    let auth = auth.unwrap();
    if auth.is_empty() {
        return Ok(None);
    }

    if !auth.starts_with("Bearer ") {
        return Err(TinyBoardsError::new(400, String::from("Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`")));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = String::from(master_key.jwt.clone());

    blocking(pool, |conn| User::from_jwt(conn, token, master_key)).await?
}

/// Enforces a logged in user. Returns `Ok<User>` if everything is OK, otherwise errors. If being logged in is optional, `load_user_opt` should be used.
pub async fn require_user(
    pool: &PgPool,
    master_key: &Secret,
    auth: Option<&str>,
) -> Result<User, TinyBoardsError> {
    if auth.is_none() {
        return Err(TinyBoardsError::err_401());
    }

    let u = load_user_opt(pool, master_key, auth).await?;
    match u {
        Some(u) => Ok(u),
        None => Err(TinyBoardsError::err_401()),
    }
}

pub fn check_user_valid(
    banned: bool,
    ban_expires: Option<chrono::NaiveDateTime>,
    deleted: bool,
) -> Result<(), TinyBoardsError> {
    if is_banned(banned, ban_expires) {
        return Err(TinyBoardsError {
            message: String::from("site ban"),
            error_code: 401,
        });
    }

    if deleted {
        return Err(TinyBoardsError {
            message: String::from("deleted"),
            error_code: 401,
        });
    }

    Ok(())
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
        return Err(TinyBoardsError::new(400, String::from("Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`")));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = String::from(master_key.jwt.clone());

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
        return Err(TinyBoardsError::err_401());
    }

    let user_view = get_user_view_from_jwt_opt(auth, pool, master_key).await?;
    match user_view {
        Some(user_view) => Ok(user_view),
        None => Err(TinyBoardsError::err_401()),
    }
}

/* #[tracing::instrument(skip_all)]
pub async fn get_user_view_from_jwt(
    jwt: &str,
    pool: &PgPool,
    master_key: &Secret,
) -> Result<UserView, TinyBoardsError> {
    let u = require_user(pool, master_key, Some(jwt)).await?;
    let user_id = u.id;

    let user_view = blocking(pool, move |conn| {
        UserView::read(conn, user_id)
            .map_err(|_e| TinyBoardsError::from_string("could not find user", 404))
    })
    .await??;

    //check_user_valid(u.banned, u.expires, u.deleted)?;

    Ok(user_view)
}

#[tracing::instrument(skip_all)]
pub async fn get_user_view_from_jwt_opt(
    jwt: Option<&str>,
    pool: &PgPool,
    master_key: &Secret,
) -> Result<Option<UserView>, TinyBoardsError> {
    match jwt {
        Some(jwt) => Ok(Some(get_user_view_from_jwt(jwt, pool, master_key).await?)),
        None => Ok(None),
    }
} */

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
                .map_err(|_e| TinyBoardsError::from_string("could not find user registration", 404))
        })
        .await??;

        if let Some(deny_reason) = registration.deny_reason {
            let registration_denied_message = &deny_reason;
            return Err(TinyBoardsError::from_string(
                registration_denied_message,
                405,
            ));
        } else {
            return Err(TinyBoardsError::from_string(
                "registration application pending",
                401,
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
                .map_err(|_e| TinyBoardsError::from_string("could not read site", 500))
        })
        .await??;

        if !site.enable_downvotes {
            return Err(TinyBoardsError::from_string("downvotes are disabled", 405));
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_private_instance(
    user_view: &Option<UserView>,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    if user_view.is_none() {
        let site = blocking(pool, move |conn| Site::read_local(conn)).await?;

        if let Ok(site) = site {
            if site.private_instance {
                return Err(TinyBoardsError::from_string("instance is private", 405));
            }
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn is_admin(pool: &PgPool, user_id: i32) -> Result<(), TinyBoardsError> {
    let user = blocking(pool, move |conn| {
        User::read(conn, user_id)
            .map_err(|_e| TinyBoardsError::from_string("could not find user", 404))
    })
    .await??;

    if !user.admin {
        return Err(TinyBoardsError::from_string("not an admin", 405));
    }

    Ok(())
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
        return Err(TinyBoardsError::from_string("not a mod or admin", 405));
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_board_ban(
    user_id: i32,
    board_id: i32,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    let is_banned = move |conn: &mut _| BoardUserBanView::get(conn, user_id, board_id).is_ok();

    if blocking(pool, is_banned).await? {
        Err(TinyBoardsError::from_string("board banned", 405))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_board_deleted_or_removed(
    board_id: i32,
    pool: &PgPool,
) -> Result<(), TinyBoardsError> {
    let board = blocking(pool, move |conn| Board::read(conn, board_id))
        .await?
        .map_err(|_e| TinyBoardsError::from_string("couldn't find board", 404))?;

    if board.deleted || board.removed {
        Err(TinyBoardsError::from_string(
            "board deleted or removed",
            404,
        ))
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
        .map_err(|_e| TinyBoardsError::from_string("couldn't find post", 404))?;

    if post.locked {
        Err(TinyBoardsError::from_string("post locked", 405))
    } else if post.deleted || post.removed {
        Err(TinyBoardsError::from_string("post deleted or removed", 404))
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
        .map_err(|_e| TinyBoardsError::from_string("couldn't find comment", 404))?;

    if comment.deleted || comment.removed {
        Err(TinyBoardsError::from_string(
            "comment deleted or removed",
            404,
        ))
    } else {
        Ok(())
    }
}
