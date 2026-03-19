use crate::utils::DbPool;
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait Crud: Sized {
    type InsertForm;
    type UpdateForm;

    async fn create(pool: &DbPool, form: &Self::InsertForm) -> Result<Self, TinyBoardsError>;
    async fn read(pool: &DbPool, id: Uuid) -> Result<Self, TinyBoardsError>;
    async fn update(pool: &DbPool, id: Uuid, form: &Self::UpdateForm) -> Result<Self, TinyBoardsError>;
    async fn delete(pool: &DbPool, id: Uuid) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Subscribeable: Sized {
    type Form;

    async fn subscribe(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn unsubscribe(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
    async fn subscribe_accepted(
        pool: &DbPool,
        board_id: Uuid,
        user_id: Uuid,
    ) -> Result<Self, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Joinable: Sized {
    type Form;

    async fn join(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn leave(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Voteable: Sized {
    type Form;

    async fn vote(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn remove_vote(
        pool: &DbPool,
        user_id: Uuid,
        item_id: Uuid,
    ) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Bannable: Sized {
    type Form;

    async fn ban(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn unban(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Saveable: Sized {
    type Form;

    async fn save(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn unsave(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Followable: Sized {
    type Form;

    async fn follow(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn unfollow(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
    async fn accept_follow(
        pool: &DbPool,
        user_id: Uuid,
        follower_id: Uuid,
    ) -> Result<Self, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Blockable: Sized {
    type Form;

    async fn block(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn unblock(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Reportable: Sized {
    type Form;

    async fn report(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn resolve(
        pool: &DbPool,
        report_id: Uuid,
        resolver_id: Uuid,
    ) -> Result<usize, TinyBoardsError>;
    async fn dismiss(
        pool: &DbPool,
        report_id: Uuid,
        resolver_id: Uuid,
    ) -> Result<usize, TinyBoardsError>;
}

#[async_trait::async_trait]
pub trait Readable: Sized {
    type Form;

    async fn mark_as_read(pool: &DbPool, form: &Self::Form) -> Result<Self, TinyBoardsError>;
    async fn mark_as_unread(pool: &DbPool, form: &Self::Form) -> Result<usize, TinyBoardsError>;
}

pub trait JoinView {
    type JoinTuple;

    fn from_tuple(tuple: Self::JoinTuple) -> Self
    where
        Self: Sized;
}

pub trait ViewToVec {
    type DbTuple;

    fn from_tuple_to_vec(tuple: Vec<Self::DbTuple>) -> Vec<Self>
    where
        Self: Sized;
}
