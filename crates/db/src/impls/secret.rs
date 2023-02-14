use crate::models::secret::Secret;
use diesel::{result::Error, *};

impl Secret {
    /// Initialize Secrets from DB.
    /// Warning: You should call this exactly once.
    pub fn init(db_url: String) -> Result<Secret, Error> {    
        let mut conn 
            = PgConnection::establish(&db_url)
                .expect("could not establish connection");
        Self::read_secrets(&mut conn)
    }

    fn read_secrets(conn: &mut PgConnection) -> Result<Secret, Error> {
        use crate::schema::secret::dsl::*;
        secret.first::<Secret>(conn)
    }
}