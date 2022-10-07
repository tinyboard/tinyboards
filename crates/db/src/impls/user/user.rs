use crate::models::user::user::{User, UserForm};
use diesel::result::Error;
use crate::traits::Crud;
use diesel::prelude::*;
use crate::schema::user_::dsl::*;
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

    // pub fn insert(
    //     conn: &mut PgConnection,
    //     username: String,
    //     fedi_name: String,
    //     password: String,
    //     email: Option<String>,
    // ) -> Result<Self, PorplError> {
    //     use crate::schema::user_;

    //     let username = username.replace('%', "\\%").replace('_', "\\_");

    //     let email: Option<String> =
    //         email.map(|email| email.replace('%', "\\%").replace('_', "\\_"));
        
        
    //     let hash = hash_password(password);

    //     Self::check_reserved(conn, &username, &&email)?;

    //     let new_user = InsertUser {
    //         name: username,
    //         fedi_name,
    //         email,
    //         passhash: hash,
    //     };

    //     diesel::insert_into(user_::table)
    //         .values(&new_user)
    //         .get_result::<Self>(conn)
    //         .map_err(|e| {
    //             eprintln!("ERROR: {e}");
    //             PorplError::err_500()
    //         })
    // }

    pub fn register(conn: &mut PgConnection, form: &UserForm) -> Result<Self, Error> {
        let mut edited_user = form.clone();
        let phash = form
        .passhash
        .as_ref()
        .map(|p| hash_password(String::from(p)));
        edited_user.passhash = phash;

        Self::create(conn, &edited_user)
    }
}


impl Crud for User {
    type Form = UserForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, user_id: i32) -> Result<Self, Error> {
        user_.find(user_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, user_id: i32) -> Result<usize, Error> {
        diesel::delete(user_.find(user_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &UserForm) -> Result<Self, Error> {
        let local_user = diesel::insert_into(user_)
            .values(form)
            .get_result::<Self>(conn)?;
        
        Ok(local_user)
    }
    fn update(conn: &mut PgConnection, user_id: i32, form: &UserForm) -> Result<Self, Error> {
        diesel::update(user_.find(user_id))
            .set(form)
            .get_result::<Self>(conn)
    }

}