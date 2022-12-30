use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

use crate::models::user::users::{User, UserForm, UserSafe};
use crate::schema::users::dsl::*;
use crate::traits::Crud;
use crate::utils::naive_now;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::PgConnection;
use tinyboards_utils::{hash_password, TinyBoardsError};

impl User {
    pub fn check_name_and_email(
        conn: &mut PgConnection,
        username: &str,
        emailaddr: &Option<String>,
    ) -> Result<(), TinyBoardsError> {
        use crate::schema::users::dsl::*;

        let user = if let Some(emailaddr) = emailaddr {
            users
                .select(id)
                .filter(name.ilike(username))
                .or_filter(email.ilike(emailaddr))
                .first::<i32>(conn)
        } else {
            users
                .select(id)
                .filter(name.ilike(username))
                .first::<i32>(conn)
        }
        .optional()?;

        if user.is_some() {
            return Err(TinyBoardsError::from_message(
                409,
                "username or email was already taken",
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
    ) -> Result<Option<Self>, TinyBoardsError> {
        use crate::schema::users::dsl::*;

        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key)?;

        let uid = claims["uid"].parse::<i32>()?;

        users
            .filter(id.eq(uid))
            .first::<Self>(conn)
            .optional()
            .map_err(|e| TinyBoardsError::from_error_message(e, 401, "error getting user from jwt"))
    }
    pub fn update_ban(
        conn: &mut PgConnection,
        user_id: i32,
        new_banned: bool,
    ) -> Result<Self, Error> {
        //use crate::schema::user::dsl::*;
        diesel::update(users.find(user_id))
            .set((is_banned.eq(new_banned), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_admin(
        conn: &mut PgConnection,
        user_id: i32,
        new_admin: bool,
    ) -> Result<Self, Error> {
        use crate::schema::users::dsl::*;
        diesel::update(users.find(user_id))
            .set((is_admin.eq(new_admin), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn get_by_name(conn: &mut PgConnection, username: &str) -> Result<Self, Error> {
        use crate::schema::users::dsl::*;
        // sanitization could be better
        users
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
        use crate::schema::users::dsl::*;
        users
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

    pub fn register(conn: &mut PgConnection, form: UserForm) -> Result<Self, TinyBoardsError> {
        Self::check_name_and_email(conn, &form.name.clone().unwrap_or_default(), &form.email)?;

        let unencrypted = form.passhash.unwrap();

        // hash the password here
        let form = UserForm {
            passhash: Some(hash_password(unencrypted)),
            ..form
        };

        Self::create(conn, &form)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not create user"))
    }

    pub fn has_active_ban(&self) -> bool {
        if let Some(expires_) = self.unban_date {
            self.is_banned && expires_.gt(&chrono::prelude::Utc::now().naive_utc())
        } else {
            self.is_banned
        }
    }

    pub fn update_settings(
        conn: &mut PgConnection,
        user_id: i32,
        form: &UserForm,
    ) -> Result<Self, Error> {
        diesel::update(users.find(user_id))
            .set(form)
            .get_result::<Self>(conn)
    }

    pub fn into_safe(self) -> UserSafe {
        UserSafe {
            id: self.id,
            name: self.name,
            preferred_name: self.preferred_name,
            is_admin: self.is_admin,
            is_banned: self.is_banned,
            creation_date: self.creation_date,
            updated: self.updated,
            theme: self.theme,
            default_sort_type: self.default_sort_type,
            default_listing_type: self.default_listing_type,
            avatar: self.avatar,
            email: self.email,
            email_notifications_enabled: self.email_notifications_enabled,
            show_nsfw: self.show_nsfw,
            is_deleted: self.is_deleted,
            unban_date: self.unban_date,
            banner: self.banner,
            bio: self.bio,
            is_application_accepted: self.is_application_accepted,
        }
    }
}

impl Crud for User {
    type Form = UserForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, user_id: i32) -> Result<Self, Error> {
        users.find(user_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, user_id: i32) -> Result<usize, Error> {
        diesel::delete(users.find(user_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &UserForm) -> Result<Self, Error> {
        let local_user = diesel::insert_into(users)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(local_user)
    }
    fn update(conn: &mut PgConnection, user_id: i32, form: &UserForm) -> Result<Self, Error> {
        diesel::update(users.find(user_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}

pub mod safe_type {
    use crate::{
        models::user::users::{UserSafe, UserSettings},
        schema::users::*,
        traits::ToSafe,
    };

    type Columns = (
        id,
        name,
        preferred_name,
        is_admin,
        is_banned,
        creation_date,
        updated,
        theme,
        default_sort_type,
        default_listing_type,
        avatar,
        email,
        email_notifications_enabled,
        show_nsfw,
        is_deleted,
        unban_date,
        banner,
        bio,
        is_application_accepted,
    );

    type SettingColumns = (
        id,
        email,
        show_nsfw,
        theme,
        default_sort_type,
        default_listing_type,
        email_notifications_enabled,
        avatar,
        banner,
        bio,
    );

    impl ToSafe for UserSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                preferred_name,
                is_admin,
                is_banned,
                creation_date,
                updated,
                theme,
                default_sort_type,
                default_listing_type,
                avatar,
                email,
                email_notifications_enabled,
                show_nsfw,
                is_deleted,
                unban_date,
                banner,
                bio,
                is_application_accepted,
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
                avatar,
                banner,
                bio,
            )
        }
    }
}
