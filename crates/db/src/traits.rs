use crate::{newtypes::UserId, utils::DbPool};
use async_trait::async_trait;
use diesel::{result::Error, PgConnection};
use tinyboards_utils::TinyBoardsError;


#[async_trait]
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

pub trait Subscribeable {
    type Form;
    fn subscribe(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
    where
        Self: Sized;
    fn unsubscribe(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
    where
        Self: Sized;
}

pub trait Joinable {
    type Form;
    fn join(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    fn leave(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
}

pub trait Voteable {
    type Form;
    type IdType;
    fn vote(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    fn remove(
        conn: &mut PgConnection,
        user_id: i32,
        item_id: Self::IdType,
    ) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

pub trait Bannable {
    type Form;
    fn ban(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error>
    where
        Self: Sized;
    fn unban(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, Error>
    where
        Self: Sized;
}

pub trait Saveable {
    type Form;
    fn save(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    fn unsave(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

pub trait Blockable {
    type Form;
    fn block(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    fn unblock(conn: &mut PgConnection, form: &Self::Form) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

pub trait Readable {
    type Form;
    fn mark_as_read(conn: &mut PgConnection, form: &Self) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    fn mark_as_unread(conn: &mut PgConnection, form: &Self) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
}

pub trait Reportable {
    type Form;
    type IdType;
    fn report(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, TinyBoardsError>
    where
        Self: Sized;
    fn resolve(
        conn: &mut PgConnection,
        report_id: Self::IdType,
        resolver_id: UserId,
    ) -> Result<usize, TinyBoardsError>
    where
        Self: Sized;
    fn unresolve(
        conn: &mut PgConnection,
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

pub trait Moderateable {
    fn get_board_id(&self) -> i32;
    fn remove(
        &self,
        admin_id: Option<i32>,
        reason: Option<String>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError>;
    fn approve(
        &self,
        admin_id: Option<i32>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError>;
    fn lock(&self, admin_id: Option<i32>, conn: &mut PgConnection) -> Result<(), TinyBoardsError>;
    fn unlock(&self, admin_id: Option<i32>, conn: &mut PgConnection)
        -> Result<(), TinyBoardsError>;
}
