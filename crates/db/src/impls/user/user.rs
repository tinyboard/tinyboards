use crate::models::user::user::{User, InsertUser};
use diesel::prelude::*;
use diesel::PgConnection;
use porpl_utils::{ 
    PorplError, 
    hash_password
};

impl User {
    fn check_reserved(
        conn: &mut PgConnection,
        uname: &str,
        emailaddr: &Option<&str>,
    ) -> Result<(), PorplError> {
        use crate::schema::user_::dsl::*;

        let user = if let Some(emailaddr) = emailaddr {
            user_
                .select(id)
                .filter(name.ilike(uname))
                .or_filter(email.ilike(emailaddr))
                .first::<i32>(conn)
        } else {
            user_
                .select(id)
                .filter(name.ilike(uname))
                .first::<i32>(conn)
        }
        .optional()
        .map_err(|e| {
            eprintln!("ERROR: {e}");
            PorplError::err_500()
        })?;

        if user.is_some() {
            return Err(PorplError::new(
                400,
                String::from("Username/Email already taken!"),
            ));
        }

        Ok(())
    }

    pub fn insert(
        conn: &mut PgConnection,
        username: String,
        fedi_name: String,
        password: String,
        email: Option<String>,
    ) -> Result<Self, PorplError> {
        use crate::schema::user_;

        let username = username.replace('%', "\\%").replace('_', "\\_");

        let email: Option<String> =
            email.map(|email| email.replace('%', "\\%").replace('_', "\\_"));
        
        
        let hash = hash_password(password);

        Self::check_reserved(conn, &username, &&email)?;

        let new_user = InsertUser {
            name: username,
            fedi_name,
            email,
            passhash: hash,
        };

        diesel::insert_into(user_::table)
            .values(&new_user)
            .get_result::<Self>(conn)
            .map_err(|e| {
                eprintln!("ERROR: {e}");
                PorplError::err_500()
            })
    }


}