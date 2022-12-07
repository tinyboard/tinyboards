use diesel::{result::Error, *};
use crate::{
    models::board::{
        board_moderator::{BoardModerator, BoardModeratorForm},
    },
    traits::Crud,
};

impl BoardModerator {
    pub fn remove_board_mod(conn: &mut PgConnection, form: &BoardModeratorForm) -> Result<usize, Error> {
        use crate::schema::board_moderator::dsl::*;
        diesel::delete(
            board_moderator
                .filter(board_id.eq(form.board_id))
                .filter(user_id.eq(form.user_id))
        )
        .execute(conn)
    }
}

impl Crud for BoardModerator {
    type Form = BoardModeratorForm;
    type IdType = i32;
    fn read(conn: &mut PgConnection, id_: i32) -> Result<Self, Error> {
        use crate::schema::board_moderator::dsl::*;
        board_moderator.find(id_).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, id_: i32) -> Result<usize, Error> {
        use crate::schema::board_moderator::dsl::*;
        diesel::delete(board_moderator.find(id_)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::board_moderator::dsl::*;
        let new = diesel::insert_into(board_moderator)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new)
    }
    fn update(conn: &mut PgConnection, id_: i32, form: &Self::Form) -> Result<Self, Error> {
        use crate::schema::board_moderator::dsl::*;
        diesel::update(board_moderator.find(id_))
            .set(form)
            .get_result::<Self>(conn)
    }
}