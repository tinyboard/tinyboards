use actix_web::web;
use porpl_db::database::PgPool;

use porpl_utils::PorplError;

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
