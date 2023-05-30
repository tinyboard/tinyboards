use crate::schema::user_comment_save::dsl::*;
use crate::utils::{DbPool, get_conn};
use crate::{
    models::comment::comment_saved::{CommentSaved, CommentSavedForm},
    traits::Saveable,
};
use diesel::prelude::*;
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Saveable for CommentSaved {
    type Form = CommentSavedForm;

    async fn save(pool: &DbPool, form: &CommentSavedForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(user_comment_save)
            .values(form)
            .on_conflict((comment_id, person_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not save comment"))
    }

    async fn unsave(pool: &DbPool, form: &CommentSavedForm) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            user_comment_save
                .filter(comment_id.eq(form.comment_id))
                .filter(person_id.eq(form.person_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unsave comment"))
    }
}