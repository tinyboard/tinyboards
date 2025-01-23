use crate::{
    schema::language::dsl::{code, id, language},
    models::apub::language::Language,
    utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl, ExpressionMethods,};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl Language {
    pub async fn read_all(pool: &DbPool) -> Result<Vec<Language>, Error> {
        let conn = &mut get_conn(pool).await?;
        Self::read_all_conn(conn).await
    }

    pub async fn read_all_conn(conn: &mut AsyncPgConnection) -> Result<Vec<Language>, Error> {
        language.load(conn).await
    }

    pub async fn read_from_id(pool: &DbPool, id_: i32) -> Result<Language, Error> {
        let conn = &mut get_conn(pool).await?;
        language.filter(id.eq(id_)).first::<Self>(conn).await
    }

    /// attempts to find the language id from the language code, if not found return none.
    pub async fn read_id_from_code(
        pool: &DbPool,
        code_: Option<&str>
    ) -> Result<Option<i32>, Error> {
        if let Some(code_) = code_ {
            let conn = &mut get_conn(pool).await?;
            Ok(
                language
                    .filter(code.eq(code_))
                    .first::<Self>(conn)
                    .await
                    .map(|l| l.id)
                    .ok(),
            )
        } else {
            Ok(None)
        }
    }
}