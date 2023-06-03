use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

use crate::models::person::local_user::{LocalUser, LocalUserForm, LocalUserSafe};
use crate::schema::local_user::dsl::*;
use crate::traits::Crud;
use crate::utils::{naive_now, fuzzy_search, DbPool, get_conn};
use diesel::{prelude::*, result::Error};
use tinyboards_utils::{hash_password, TinyBoardsError};
use diesel_async::RunQueryDsl;

impl LocalUser {
    pub async fn check_name_and_email(
        pool: &DbPool,
        username: &str,
        emailaddr: &Option<String>,
    ) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;

        let user = if let Some(emailaddr) = emailaddr {
            local_user
                .select(id)
                .filter(name.ilike(username))
                .or_filter(email.ilike(emailaddr))
                .first::<i32>(conn)
                .await
        } else {
            local_user
                .select(id)
                .filter(name.ilike(username))
                .first::<i32>(conn)
                .await
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

    pub async fn from_jwt(
        pool: &DbPool,
        token: String,
        master_key: String,
    ) -> Result<Option<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;

        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key)?;

        let uid = claims["uid"].parse::<i32>()?;

        local_user
            .filter(id.eq(uid))
            .first::<Self>(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::from_error_message(e, 401, "error getting user from jwt"))
    }

    pub async fn update_ban(
        pool: &DbPool,
        id_: i32,
        new_banned: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user.find(id_))
            .set((is_banned.eq(new_banned), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_passhash(
        pool: &DbPool,
        id_: i32,
        new_passhash: String,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user.find(id_))
            .set((passhash.eq(new_passhash), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_is_application_accepted(
        pool: &DbPool,
        id_: i32,
        new_is_application_accepted: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user.find(id_))
            .set((is_application_accepted.eq(new_is_application_accepted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn search_by_name(
        pool: &DbPool,
        query: &str
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        local_user.filter(name.ilike(fuzzy_search(query))).load(conn)
        .await
    }

    pub async fn update_admin(
        pool: &DbPool,
        id_: i32,
        new_admin: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        diesel::update(local_user.find(id_))
            .set((is_admin.eq(new_admin), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn get_by_name(pool: &DbPool, username: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        // sanitization could be better
        local_user
            .filter(
                name.ilike(
                    username
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
            .await
    }

    pub async fn get_by_person_id(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        local_user
            .filter(person_id.eq(id_))
            .first::<Self>(conn)
            .await
    }

    pub async fn get_by_email(pool: &DbPool, email_addr: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        local_user
            .filter(
                email.ilike(
                    email_addr
                        .replace(' ', "")
                        .replace('%', "\\%")
                        .replace('_', "\\_"),
                ),
            )
            .first::<Self>(conn)
            .await
    }

    pub async fn register(pool: &DbPool, form: LocalUserForm) -> Result<Self, TinyBoardsError> {
        let email_addr = &form.email.unwrap();
        Self::check_name_and_email(pool, &form.name.clone().unwrap_or_default(), &email_addr.clone()).await?;

        let unencrypted = form.passhash.unwrap();

        // hash the password here
        let form = LocalUserForm {
            passhash: Some(hash_password(unencrypted)),
            email: Some(email_addr.clone()),
            ..form
        };

        Self::create(pool, &form)
            .await    
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not create user"))
    }

    pub fn has_active_ban(&self) -> bool {
        if let Some(expires_) = self.unban_date {
            self.is_banned && expires_.gt(&chrono::prelude::Utc::now().naive_utc())
        } else {
            self.is_banned
        }
    }

    pub async fn update_settings(
        pool: &DbPool,
        id_: i32,
        form: &LocalUserForm,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }

    /// accept all users that are unaccepted, NOTE: this is only called when toggling application mode on/off
    pub async fn accept_all_applications(pool: &DbPool) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        diesel::update(local_user)
            .filter(is_application_accepted.eq(false))
            .set((is_application_accepted.eq(true), updated.eq(naive_now())))
            .execute(conn)
            .await
    }

    pub fn into_safe(self) -> LocalUserSafe {
        LocalUserSafe {
            id: self.id,
            name: self.name,
            is_admin: self.is_admin,
            is_banned: self.is_banned,
            is_deleted: self.is_deleted,
            creation_date: self.creation_date,
            updated: self.updated,
            unban_date: self.unban_date,
            theme: self.theme,
            default_sort_type: self.default_sort_type,
            default_listing_type: self.default_listing_type,
            email_notifications_enabled: self.email_notifications_enabled,
            show_nsfw: self.show_nsfw,
            show_bots: self.show_bots,
            is_application_accepted: self.is_application_accepted,
        }
    }
}

#[async_trait::async_trait]
impl Crud for LocalUser {
    type Form = LocalUserForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        local_user.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(local_user.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &LocalUserForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_local_user = diesel::insert_into(local_user)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_local_user)
    }
    async fn update(pool: &DbPool, id_: i32, form: &LocalUserForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(local_user.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

pub mod safe_type {
    use crate::{
        models::person::local_user::{LocalUserSettings, LocalUserSafe},
        schema::local_user::*,
        traits::ToSafe,
    };

    type Columns = (
        id,
        name,
        is_admin,
        is_banned,
        creation_date,
        updated,
        theme,
        default_sort_type,
        default_listing_type,
        email,
        email_notifications_enabled,
        show_nsfw,
        is_deleted,
        unban_date,
        is_application_accepted,
    );

    type SettingColumns = (
        id,
        name,
        email,
        show_nsfw,
        show_bots,
        theme,
        default_sort_type,
        default_listing_type,
        email_notifications_enabled,
        lang,
        updated,
    );

    impl ToSafe for LocalUserSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                is_admin,
                is_banned,
                creation_date,
                updated,
                theme,
                default_sort_type,
                default_listing_type,
                email,
                email_notifications_enabled,
                show_nsfw,
                is_deleted,
                unban_date,
                is_application_accepted,
            )
        }
    }

    impl ToSafe for LocalUserSettings {
        type SafeColumns = SettingColumns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                name,
                email,
                show_nsfw,
                show_bots,
                theme,
                default_sort_type,
                default_listing_type,
                email_notifications_enabled,
                lang,
                updated,
            )
        }
    }
}
