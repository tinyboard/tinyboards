use crate::schema::user_comment_save::dsl::*;
use crate::{
    models::comment::user_comment_save::{CommentSaved, CommentSavedForm},
    traits::Saveable,
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;

impl Saveable for CommentSaved {
    type Form = CommentSavedForm;

    fn save(conn: &mut PgConnection, form: &CommentSavedForm) -> Result<Self, TinyBoardsError> {
        diesel::insert_into(user_comment_save)
            .values(form)
            .on_conflict((comment_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not save comment"))
    }

    fn unsave(conn: &mut PgConnection, form: &CommentSavedForm) -> Result<usize, TinyBoardsError> {
        diesel::delete(
            user_comment_save
                .filter(comment_id.eq(form.comment_id))
                .filter(user_id.eq(form.user_id)),
        )
        .execute(conn)
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unsave comment"))
    }
}
