use crate::{
    models::post::post_saved::{PostSaved, PostSavedForm},
    traits::Saveable, utils::{get_conn, DbPool},
};
use diesel::{insert_into, prelude::*};
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

#[async_trait::async_trait]
impl Saveable for PostSaved {
    type Form = PostSavedForm;
    async fn save(pool: &DbPool, form: &PostSavedForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::user_post_save::dsl::*;
        insert_into(user_post_save)
            .values(form)
            .on_conflict((post_id, person_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not save post"))
    }

    async fn unsave(pool: &DbPool, form: &PostSavedForm) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::user_post_save::dsl::*;
        diesel::delete(
            user_post_save
                .filter(post_id.eq(form.post_id))
                .filter(person_id.eq(form.person_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unsave post"))
    }
}
