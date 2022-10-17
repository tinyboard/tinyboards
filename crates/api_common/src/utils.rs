use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token};
use porpl_db_views::structs::{UserView, BoardView, BoardUserBanView};
use sha2::Sha384;
use std::collections::BTreeMap;
use porpl_utils::error::PorplError;
use actix_web::web;
use porpl_db::{
    database::PgPool,
    impls::user::is_banned, 
    models::{
        user::user::User,
        board::board::Board, secret::Secret,
    },
    traits::Crud,
};
//use diesel::PgConnection;

pub fn get_jwt(uid: i32, uname: &str, master_key: &str) -> String {
    let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
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
 * Takes a reference to a connection pool as one argument (`PorplContext::pool(&self)`) and a closure as another.
 * Passes a mutable reference to an open connection to the db to the closure and executes it. Then, the result of the closure is returned.
 * *(shamelessly stolen from lemmy)*
 */
pub async fn blocking<F, T>(pool: &PgPool, f: F) -> Result<T, PorplError>
where
    F: FnOnce(&mut diesel::PgConnection) -> T + Send + 'static,
    T: Send + 'static,
{
    let pool = pool.clone();
    let res = web::block(move || {
        let mut conn = pool.get().unwrap();
        let res = (f)(&mut conn);
        Ok(res) as Result<T, PorplError>
    })
    .await
    .map_err(|_| PorplError::new(500, String::from("Internal Server Error (BlockingError)")))?;

    res
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), PorplError> {
    if !(10..=60).contains(&pass.len()) {
        Err(PorplError { message: String::from("invalid password"), error_code: 400 })
    } else {
      Ok(())
    }
  }

// Tries to take the access token from the auth header and get the user. Returns `Err` if it encounters an error (db error or invalid header format), otherwise `Ok(Some<User>)` or `Ok(None)` is returned depending on whether the token is valid. If being logged in is required, `require_user` should be used.
pub async fn load_user_opt(
    pool: &PgPool,
    master_key: &str,
    auth: Option<&str>,
) -> Result<Option<User>, PorplError> {
    if auth.is_none() {
        return Ok(None);
    };

    // here it is safe to unwrap, because the above check ensures that `auth` isn't None here
    let auth = auth.unwrap();
    if !auth.starts_with("Bearer ") {
        return Err(PorplError::new(400, String::from("Invalid `Authorization` header! It should follow this pattern: `Authorization: Bearer <access token>`")));
    }
    // Reference to the string stored in `auth` skipping the `Bearer ` part
    // this part makes me cringe so much, I don't want all these to be owned, but they have to be sent to another thread and the references are valid only here
    // maybe there's a better solution to this but I feel like this is too memory-consuming.
    let token = String::from(&auth[7..]);
    let master_key = String::from(master_key);

    blocking(pool, |conn| User::from_jwt(conn, token, master_key)).await?
}

/// Enforces a logged in user. Returns `Ok<User>` if everything is OK, otherwise errors. If being logged in is optional, `load_user_opt` should be used.
pub async fn require_user(
    pool: &PgPool,
    master_key: &str,
    auth: Option<&str>,
) -> Result<User, PorplError> {
    if auth.is_none() {
        return Err(PorplError::err_401());
    }

    let u = load_user_opt(pool, master_key, auth).await?;
    match u {
        Some(u) => Ok(u),
        None => Err(PorplError::err_401()),
    }
}

pub fn check_user_valid(
    banned: bool,
    ban_expires: Option<chrono::NaiveDateTime>,
    deleted: bool,  
) -> Result<(), PorplError> {
    if is_banned(banned, ban_expires) {
        return Err(PorplError { message: String::from("site ban"), error_code: 401 });
    }

    if deleted {
        return Err(PorplError { message: String::from("deleted"), error_code: 401 });
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn get_user_view_from_jwt(
  secret: &Secret,
  pool: &PgPool,
  master_key: &str,
) -> Result<UserView, PorplError> {

    let u = require_user(pool, master_key, Some(&secret.jwt)).await?;
    let user_id = u.id;

    let user_view = 
        blocking(pool, move |conn| {
            UserView::read(conn, user_id)
                .map_err(|e| {
                    eprintln!("ERROR: {}", e);
                    PorplError::err_500()
                })
        }).await??;
    
    check_user_valid(
        u.banned,
        u.expires,
        u.deleted,
    )?;

  Ok(user_view)
}

#[tracing::instrument(skip_all)]
pub async fn is_mod_or_admin(
    pool: &PgPool,
    user_id: i32,
    board_id: i32,
) -> Result<(), PorplError> {
    let is_mod_or_admin = blocking(pool, move |conn| {
        BoardView::is_mod_or_admin(conn, user_id, board_id)
    })
    .await?;

    if !is_mod_or_admin {
        return Err(PorplError::from_string("not a mod or admin", 405));
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn check_board_ban(
    user_id: i32,
    board_id: i32,
    pool: &PgPool
) -> Result<(), PorplError> {
    let is_banned =
        move |conn: &mut _| BoardUserBanView::get(conn, user_id, board_id).is_ok();
    
    if blocking(pool, is_banned).await? {
        Err(PorplError::from_string("board banned", 405))
    } else {
        Ok(())
    }
}

#[tracing::instrument(skip_all)]
pub async fn check_board_deleted_or_removed(
    board_id: i32,
    pool: &PgPool,
) -> Result<(), PorplError> {
    let board = blocking(pool, move |conn| Board::read(conn, board_id))
        .await?
        .map_err(|_e| PorplError::from_string("couldn't find board", 404))?;
    
    if board.deleted || board.removed {
        Err(PorplError::from_string("deleted", 404))
    } else {
        Ok(())
    }
}