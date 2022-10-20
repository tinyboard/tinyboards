use crate::schema::comment_saved::dsl::*;
use diesel::prelude::*;
use porpl_utils::PorplError;
use crate::{
    models::comment::comment_saved::{CommentSaved, CommentSavedForm},
    traits::Saveable,  
};

impl Saveable for CommentSaved {
    type Form = CommentSavedForm;
    
    fn save(conn: &mut PgConnection, form: &CommentSavedForm) -> Result<Self, PorplError> {
        diesel::insert_into(comment_saved)
            .values(form)
            .on_conflict((comment_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .map_err(|_| PorplError::err_500())
    }

    fn unsave(conn: &mut PgConnection, form: &CommentSavedForm) -> Result<usize, PorplError> {
        diesel::delete(
        comment_saved
                .filter(comment_id.eq(form.comment_id))
                .filter(user_id.eq(form.user_id))
        )
        .execute(conn)
        .map_err(|_| PorplError::err_500())
    }
}