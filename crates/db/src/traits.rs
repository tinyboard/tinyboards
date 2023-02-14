use crate::{newtypes::UserId, utils::DbPool};
use diesel::{result::Error};
use tinyboards_utils::TinyBoardsError;


#[async_trait::async_trait]
pub trait Crud {
    type Form;
    type IdType;

    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error>
    where
        Self: Sized;

    async fn read(pool: &DbPool, id: Self::IdType) -> Result<Self, Error>
    where
        Self: Sized;

    async fn update(pool: &DbPool, id: Self::IdType, form: &Self::Form) -> Result<Self, Error>
    where
        Self: Sized;
    async fn delete(pool: &DbPool, id: Self::IdType) -> Result<usize, Error>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Subscribeable {
    type Form;
    async fn subscribe(pool: &DbPool, form: &Self::Form) -> Result<Self, Error>
    where
        Self: Sized;
    async fn unsubscribe(pool: &DbPool, form: &Self::Form) -> Result<usize, Error>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Joinable {
    type Form;
    async fn join(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    async fn leave(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Voteable {
    type Form;
    type IdType;
    async fn vote(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    async fn remove(
        pool: &DbPool,
        user_id: i32,
        item_id: Self::IdType,
    ) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Bannable {
    type Form;
    async fn ban(pool: &DbPool, form: &Self::Form) -> Result<Self, Error>
    where
        Self: Sized;
    async fn unban(pool: &DbPool, form: &Self::Form) -> Result<usize, Error>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Saveable {
    type Form;
    async fn save(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    async fn unsave(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Blockable {
    type Form;
    async fn block(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    async fn unblock(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Readable {
    type Form;
    async fn mark_as_read(pool: &DbPool, form: &Self) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    async fn mark_as_unread(pool: &DbPool, form: &Self) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Reportable {
    type Form;
    type IdType;
    async fn report(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    async fn resolve(
        pool: &DbPool,
        report_id: Self::IdType,
        resolver_id: UserId,
    ) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
    async fn unresolve(
        pool: &DbPool,
        report_id: Self::IdType,
        resolver_id: UserId,
    ) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}
pub trait ToSafe {
    type SafeColumns;
    fn safe_columns_tuple() -> Self::SafeColumns;
}

pub trait ViewToVec {
    type DbTuple;
    fn from_tuple_to_vec(tuple: Vec<Self::DbTuple>) -> Vec<Self>
    where
        Self: Sized;
}

#[async_trait::async_trait]
pub trait Moderateable {
    fn get_board_id(&self) -> i32;
    async fn remove(
        &self,
        admin_id: Option<i32>,
        reason: Option<String>,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError>;
    async fn approve(
        &self,
        admin_id: Option<i32>,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError>;
    async fn lock(&self, admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError>;
    async fn unlock(&self, admin_id: Option<i32>, pool: &DbPool)
        -> Result<(), TinyBoardsError>;
}
