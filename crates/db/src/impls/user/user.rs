use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

use crate::models::user::user::{User, UserForm, UserSafe};
use crate::schema::user_::dsl::*;
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
        .optional()?;

        if user.is_some() {
            return Err(TinyBoardsError::from_message("username or email was already taken"));
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
        use crate::schema::user_::dsl::*;

        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key)?;

        let uid = claims["uid"]
            .parse::<i32>()?;

        user_
            .filter(id.eq(uid))
            .first::<Self>(conn)
            .optional()
            .map_err(|e| TinyBoardsError::from_error_message(e, "error getting user from jwt"))
    }
    pub fn update_ban(
        conn: &mut PgConnection,
        user_id: i32,
        new_banned: bool,
    ) -> Result<Self, Error> {
        //use crate::schema::user::dsl::*;
        diesel::update(user_.find(user_id))
            .set((banned.eq(new_banned), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_admin(
        conn: &mut PgConnection,
        user_id: i32,
        new_admin: bool,
    ) -> Result<Self, Error> {
        use crate::schema::user_::dsl::*;
        diesel::update(user_.find(user_id))
            .set((admin.eq(new_admin), updated.eq(naive_now())))
            .get_result::<Self>(conn)
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

    pub fn register(conn: &mut PgConnection, form: UserForm) -> Result<Self, TinyBoardsError> {
        Self::check_name_and_email(conn, &form.name, &form.email)?;

        // hash the password here
        let form = UserForm {
            passhash: hash_password(form.passhash),
            ..form
        };

        Self::create(conn, &form).map_err(|e| TinyBoardsError::from_error_message(e, "could not create user"))
    }

    pub fn has_active_ban(&self) -> bool {
        if let Some(expires_) = self.expires {
            self.banned && expires_.gt(&chrono::prelude::Utc::now().naive_utc())
        } else {
            self.banned
        }
    }

    pub fn into_safe(self) -> UserSafe {
        UserSafe {
            id: self.id,
            name: self.name,
            preferred_name: self.preferred_name,
            admin: self.admin,
            banned: self.banned,
            published: self.published,
            updated: self.updated,
            theme: self.theme,
            default_sort_type: self.default_sort_type,
            default_listing_type: self.default_listing_type,
            avatar: self.avatar,
            email: self.email,
            email_notifications_enabled: self.email_notifications_enabled,
            show_nsfw: self.show_nsfw,
            deleted: self.deleted,
            expires: self.expires,
            banner: self.banner,
            bio: self.bio,
            application_accepted: self.application_accepted,
        }
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
    use crate::{
        models::user::user::{UserSafe, UserSettings},
        schema::user_::*,
        traits::ToSafe,
    };

    type Columns = (
        id,
        name,
        preferred_name,
        admin,
        banned,
        published,
        updated,
        theme,
        default_sort_type,
        default_listing_type,
        avatar,
        email,
        email_notifications_enabled,
        show_nsfw,
        deleted,
        expires,
        banner,
        bio,
        application_accepted,
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
                admin,
                banned,
                published,
                updated,
                theme,
                default_sort_type,
                default_listing_type,
                avatar,
                email,
                email_notifications_enabled,
                show_nsfw,
                deleted,
                expires,
                banner,
                bio,
                application_accepted,
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
