use crate::{
    models::post::user_post_save::{PostSaved, PostSavedForm},
    traits::Saveable,
};
use diesel::{insert_into, prelude::*, PgConnection};
use tinyboards_utils::TinyBoardsError;

impl Saveable for PostSaved {
    type Form = PostSavedForm;
    fn save(conn: &mut PgConnection, form: &PostSavedForm) -> Result<Self, TinyBoardsError> {
        use crate::schema::user_post_save::dsl::*;
        insert_into(user_post_save)
            .values(form)
            .on_conflict((post_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not save post"))
    }

    fn unsave(conn: &mut PgConnection, form: &PostSavedForm) -> Result<usize, TinyBoardsError> {
        use crate::schema::user_post_save::dsl::*;
        diesel::delete(
            user_post_save
                .filter(post_id.eq(form.post_id))
                .filter(user_id.eq(form.user_id)),
        )
        .execute(conn)
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unsave post"))
    }
}
