use crate::{
    models::post::post_saved::{PostSaved, PostSavedForm},
    traits::Saveable,
    utils::{get_conn, DbPool},
};
use diesel::{insert_into, prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl PostSaved {
    /// Takes a list of post ids, and for each post, return whether it's saved or not
    pub async fn get_saved_for_ids(
        pool: &DbPool,
        ids: Vec<i32>,
        for_person_id: i32,
    ) -> Result<Vec<(i32, bool)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{post_saved, posts};

        posts::table
            .left_join(
                post_saved::table.on(post_saved::post_id
                    .eq(posts::id)
                    .and(post_saved::person_id.eq(for_person_id))),
            )
            .filter(posts::id.eq_any(ids))
            .select((posts::id, post_saved::id.nullable()))
            .load::<(i32, Option<i32>)>(conn)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(post_id, save_id)| (post_id, save_id.is_some()))
                    .collect::<Vec<(i32, bool)>>()
            })
    }
}

#[async_trait::async_trait]
impl Saveable for PostSaved {
    type Form = PostSavedForm;
    async fn save(pool: &DbPool, form: &PostSavedForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::post_saved::dsl::*;
        insert_into(post_saved)
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
        use crate::schema::post_saved::dsl::*;
        diesel::delete(
            post_saved
                .filter(post_id.eq(form.post_id))
                .filter(person_id.eq(form.person_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unsave post"))
    }
}
