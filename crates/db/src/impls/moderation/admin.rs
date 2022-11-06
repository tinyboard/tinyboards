use crate::{models::moderator::admin_actions::*, traits::Crud};
use diesel::{result::Error, *};

impl Crud for AdminPurgeBoard {
    type Form = AdminPurgeBoardForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        admin_purge_board.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        diesel::delete(admin_purge_board.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        let new = diesel::insert_into(admin_purge_board)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_board::dsl::*;
        diesel::update(admin_purge_board.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for AdminPurgeComment {
    type Form = AdminPurgeCommentForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        admin_purge_comment.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        diesel::delete(admin_purge_comment.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        let new = diesel::insert_into(admin_purge_comment)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_comment::dsl::*;
        diesel::update(admin_purge_comment.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for AdminPurgePost {
    type Form = AdminPurgePostForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        admin_purge_post.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        diesel::delete(admin_purge_post.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        let new = diesel::insert_into(admin_purge_post)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_post::dsl::*;
        diesel::update(admin_purge_post.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Crud for AdminPurgeUser {
    type Form = AdminPurgeUserForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::admin_purge_user::dsl::*;
        admin_purge_user.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::admin_purge_user::dsl::*;
        diesel::delete(admin_purge_user.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_user::dsl::*;
        let new = diesel::insert_into(admin_purge_user)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::admin_purge_user::dsl::*;
        diesel::update(admin_purge_user.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}