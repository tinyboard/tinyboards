use crate::{models::moderator::admin_actions::*, traits::Crud, utils::{DbPool, get_conn}};
use diesel::{result::Error, *};
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Crud for AdminBanBoard {
    type Form = AdminBanBoardForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_ban_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        admin_ban_board.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_ban_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(admin_ban_board.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_ban_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        let new = diesel::insert_into(admin_ban_board)
            .values(form)
            .get_result::<Self>(conn).await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_ban_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(admin_ban_board.find(id_)).set(form).get_result::<Self>(conn).await
    }
}

#[async_trait::async_trait]
impl Crud for AdminPurgeBoard {
    type Form = AdminPurgeBoardForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        admin_purge_board.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(admin_purge_board.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        let new = diesel::insert_into(admin_purge_board)
            .values(form)
            .get_result::<Self>(conn).await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(admin_purge_board.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for AdminPurgeComment {
    type Form = AdminPurgeCommentForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        let conn = &mut get_conn(pool).await?;
        admin_purge_comment.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(admin_purge_comment.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        let conn = &mut get_conn(pool).await?;
        let new = diesel::insert_into(admin_purge_comment)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(admin_purge_comment.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for AdminPurgePost {
    type Form = AdminPurgePostForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        let conn = &mut get_conn(pool).await?;
        admin_purge_post.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(admin_purge_post.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        let conn = &mut get_conn(pool).await?;
        let new = diesel::insert_into(admin_purge_post)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(admin_purge_post.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for AdminPurgePerson {
    type Form = AdminPurgePersonForm;
    type IdType = i32;
    async fn read(pool: &DbPool, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_person::dsl::*;
        let conn = &mut get_conn(pool).await?;
        admin_purge_person.find(id_).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_person::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::delete(admin_purge_person.find(id_)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_person::dsl::*;
        let conn = &mut get_conn(pool).await?;
        let new = diesel::insert_into(admin_purge_person)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new)
    }
    async fn update(pool: &DbPool, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_person::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(admin_purge_person.find(id_))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}