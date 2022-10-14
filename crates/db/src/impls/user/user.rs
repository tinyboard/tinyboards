use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

use crate::models::user::user::{User, UserForm};
use crate::schema::user_::dsl::*;
use crate::traits::Crud;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use porpl_utils::{hash_password, PorplError};

impl User {
    pub fn check_name_and_email(
        conn: &mut PgConnection,
        username: &str,
        emailaddr: &Option<String>,
    ) -> Result<(), PorplError> {
        use crate::schema::user_::dsl::*;

        let user = if let Some(emailaddr) = emailaddr {
            user_
                .select(id)
                .filter(name.ilike(username))
                .or_filter(email.ilike(emailaddr))
                .first::<i32>(conn)
        } else {
            user_
                .select(id)
                .filter(name.ilike(username))
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

    pub fn get_jwt(&self, master_key: &str) -> String {
        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let header = Header {
            algorithm: AlgorithmType::Hs384,
            ..Default::default()
        };

        let mut claims = BTreeMap::new();
        claims.insert("uid", self.id.to_string());
        //claims.insert("login_nonce", self.login_nonce.to_string());

        let token = Token::new(header, claims)
            .sign_with_key(&key)
            .unwrap()
            .as_str()
            .to_string();

        token
    }

    pub fn from_jwt(
        conn: &mut PgConnection,
        token: String,
        master_key: String,
    ) -> Result<Option<Self>, PorplError> {
        use crate::schema::user_::dsl::*;

        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key).map_err(|e| {
            eprintln!("ERROR: {:#?}", e);
            PorplError::err_500()
        })?;

        let uid = claims["uid"]
            .parse::<i32>()
            .map_err(|_| PorplError::err_500())?;

        user_
            .filter(id.eq(uid))
            .first::<Self>(conn)
            .optional()
            .map_err(|e| {
                eprintln!("ERROR: {}", e);
                PorplError::err_500()
            })
    }

    pub fn get_by_name(conn: &mut PgConnection, username: &str) -> Result<Self, Error> {
        use crate::schema::user_::dsl::*;
        // sanitization could be better
        user_
            .filter(
                name.ilike(
                    username
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
    }

    pub fn get_by_email(conn: &mut PgConnection, email_addr: &str) -> Result<Self, Error> {
        use crate::schema::user_::dsl::*;
        user_
            .filter(
                email.ilike(
                    email_addr
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
    }

    pub fn register(conn: &mut PgConnection, form: UserForm) -> Result<Self, PorplError> {
        Self::check_name_and_email(conn, &form.name, &form.email)?;

        // hash the password here
        let form = UserForm {
            passhash: hash_password(form.passhash),
            ..form
        };

        Self::create(conn, &form).map_err(|e| {
            eprintln!("ERROR: {}", e);
            PorplError::new(500, String::from("Internal error, please try again later"))
        })
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

pub mod safe_type {
    use crate::{schema::user_::*, models::user::user::{UserSafe, UserSettings}, traits::ToSafe};

    type Columns = (
        id,
        name,
        fedi_name,
        preferred_name,
        admin,
        banned,
        published,
        updated,
        theme,
        default_sort_type,
        default_listing_type,
        avatar,
        email_notifications_enabled,
        show_nsfw,
    );


    type SettingColumns = (
        id,
        email,
        show_nsfw,
        theme,
        default_sort_type,
        default_listing_type,
        email_notifications_enabled,
    );


    impl ToSafe for UserSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                fedi_name,
                preferred_name,
                admin,
                banned,
                published,
                updated,
                theme,
                default_sort_type,
                default_listing_type,
                avatar,
                email_notifications_enabled,
                show_nsfw,
            )
        }
    }

    impl ToSafe for UserSettings {
        type SafeColumns = SettingColumns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                email,
                show_nsfw,
                theme,
                default_sort_type,
                default_listing_type,
                email_notifications_enabled,
            )
        }
    }
}

