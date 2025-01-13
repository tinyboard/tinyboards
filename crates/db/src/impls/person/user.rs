use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use hmac::{Hmac, Mac};
#[allow(unused_imports)]
use jwt::{AlgorithmType, Header, SignWithKey, Token, VerifyWithKey};
use sha2::Sha384;
use std::collections::BTreeMap;

use crate::{
    aggregates::structs::PersonAggregates,
    models::person::{
        local_user::{AdminPerms, LocalUser},
        person::Person,
        user::User,
    },
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

impl User {
    pub fn has_permission(&self, perm: AdminPerms) -> bool {
        match self.local_user {
            Some(ref u) => u.has_permission(perm),
            None => false,
        }
    }

    pub fn has_permissions_any(&self, perms: i32) -> bool {
        match self.local_user {
            Some(ref u) => u.has_permissions_any(perms),
            None => false,
        }
    }

    pub async fn from_jwt(
        pool: &DbPool,
        token: String,
        master_key: String,
    ) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{local_user, person, person_aggregates};

        let key: Hmac<Sha384> = Hmac::new_from_slice(master_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key)?;

        let uid = claims["uid"].parse::<i32>()?;

        person::table
            .inner_join(person_aggregates::table)
            .inner_join(local_user::table.on(local_user::person_id.eq(person::id)))
            .filter(local_user::id.eq(uid))
            .first::<(Person, PersonAggregates, LocalUser)>(conn)
            .await
            .map(|(person, counts, local)| Self {
                person,
                counts,
                local_user: Some(local),
            })
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 401, "Invalid auth token provided.")
            })
    }
}

impl From<(Person, PersonAggregates, Option<LocalUser>)> for User {
    fn from((person, counts, local_user): (Person, PersonAggregates, Option<LocalUser>)) -> Self {
        Self {
            person,
            counts,
            local_user,
        }
    }
}
