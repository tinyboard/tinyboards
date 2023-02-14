use crate::{
    models::comment::comment_reply::{CommentReply, CommentReplyForm},
    traits::Crud, utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for CommentReply {
    type Form = CommentReplyForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_reply::dsl::*;
        comment_reply.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_reply::dsl::*;
        diesel::delete(comment_reply.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_reply::dsl::*;
        let new = diesel::insert_into(comment_reply)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comment_reply::dsl::*;
        diesel::update(comment_reply.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}