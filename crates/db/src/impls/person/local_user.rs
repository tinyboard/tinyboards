use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;
use std::ops::Add;

use crate::models::person::local_user::{AdminPerms, LocalUser, LocalUserForm, LocalUserSafe};
use crate::schema::local_user::dsl::*;
use crate::traits::Crud;
use crate::utils::{fuzzy_search, get_conn, naive_now, DbPool};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::{hash_password, TinyBoardsError};

impl AdminPerms {
    pub fn as_i32(&self) -> i32 {
        use AdminPerms::*;

        match self {
            Null => 0,
            Appearance => 2,
            Config => 4,
            Content => 8,
            Users => 16,
            Boards => 32,
            Full => 64,
            Owner => 128,
            System => 256,
        }
    }
}

impl Add for AdminPerms {
    type Output = i32;

    fn add(self, other: Self) -> i32 {
        self.as_i32() + other.as_i32()
    }
}

impl LocalUser {
    /**
       Check permission using the bitwise or operator.
       Always true for full perms and the owner account.
    */
    pub fn has_permission(&self, permission: AdminPerms) -> bool {
        if (self.admin_level & AdminPerms::Owner.as_i32()) > 0
            || (self.admin_level & AdminPerms::Full.as_i32()) > 0
            || (self.admin_level & permission.as_i32()) > 0
        {
            true
        } else {
            false
        }
    }

    /**
     An alternative for `has_permission` for cases where any of the given permissions are enough.
     For example, admins who have either the Content or Boards permission (or both) can bypass board bans.
       Example usage: `u.has_permissions_any(AdminPerms::Content + AdminPerms::Boards)`
    */
    pub fn has_permissions_any(&self, permissions: i32) -> bool {
        if (self.admin_level & AdminPerms::Owner.as_i32()) > 0
            || (self.admin_level & AdminPerms::Full.as_i32()) > 0
            || (self.admin_level & permissions) > 0
        {
            true
        } else {
            false
        }
    }

    /**
     * For when only the owner has permissions to do something. (duh)
     */
    pub fn is_owner(&self) -> bool {
        self.admin_level & AdminPerms::Owner.as_i32() > 0
    }

    pub async fn get_unread_replies_count(&self, pool: &DbPool) -> Result<i64, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::notifications;

        notifications::table
            .select(notifications::id)
            .filter(
                notifications::recipient_id.eq(self.id).and(
                    notifications::kind
                        .eq("PostReply")
                        .or(notifications::kind.eq("CommentReply")),
                ),
            )
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "Failed to retrieve unread notification count",
                )
            })
    }

    pub async fn get_unread_mentions_count(&self, pool: &DbPool) -> Result<i64, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::notifications;

        notifications::table
            .select(notifications::id)
            .filter(
                notifications::recipient_id
                    .eq(self.id)
                    .and(notifications::kind.eq("UsernameMention")),
            )
            .filter(notifications::is_read.eq(false))
            .count()
            .get_result(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "Failed to retrieve unread notification count",
                )
            })
    }

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
            .set((
                is_application_accepted.eq(new_is_application_accepted),
                updated.eq(naive_now()),
            ))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn search_by_name(pool: &DbPool, query: &str) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        local_user
            .filter(name.ilike(fuzzy_search(query)))
            .load(conn)
            .await
    }

    pub async fn update_admin(
        pool: &DbPool,
        id_: i32,
        new_admin_level: i32,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::local_user::dsl::*;
        diesel::update(local_user.find(id_))
            .set((admin_level.eq(new_admin_level), updated.eq(naive_now())))
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
        Self::check_name_and_email(
            pool,
            &form.name.clone().unwrap_or_default(),
            &email_addr.clone(),
        )
        .await?;

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
            person_id: self.person_id,
            name: self.name,
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
            admin_level: self.admin_level,
        }
    }
}

#[async_trait::async_trait]
impl Crud for LocalUser {
    type Form = LocalUserForm;
    type IdType = i32;

    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        local_user.find(id_).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(local_user.find(id_)).execute(conn).await
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
        models::person::local_user::{LocalUserSafe, LocalUserSettings},
        schema::local_user::*,
        traits::ToSafe,
    };

    type Columns = (
        id,
        person_id,
        name,
        is_deleted,
        creation_date,
        updated,
        unban_date,
        theme,
        default_sort_type,
        default_listing_type,
        email_notifications_enabled,
        show_nsfw,
        show_bots,
        is_application_accepted,
        admin_level,
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
        interface_language,
        updated,
    );

    impl ToSafe for LocalUserSafe {
        type SafeColumns = Columns;

        fn safe_columns_tuple() -> Self::SafeColumns {
            (
                id,
                person_id,
                name,
                is_deleted,
                creation_date,
                updated,
                unban_date,
                theme,
                default_sort_type,
                default_listing_type,
                email_notifications_enabled,
                show_nsfw,
                show_bots,
                is_application_accepted,
                admin_level,
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
                interface_language,
                updated,
            )
        }
    }
}
