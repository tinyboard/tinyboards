use crate::models::secret::Secret;
use diesel::{result::Error, *};

impl Secret {
    /// Initialize Secrets from DB.
    /// Warning: You should call this exactly once.
    pub fn init(conn: &mut PgConnection) -> Result<Secret, Error> {
        Self::read_secrets(conn)
    }

    fn read_secrets(conn: &mut PgConnection) -> Result<Secret, Error> {
        use crate::schema::secret::dsl::*;
        secret.first::<Secret>(conn)
    }
}