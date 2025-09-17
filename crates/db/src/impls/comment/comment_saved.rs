use crate::schema::comment_saved::dsl::*;
use crate::utils::{get_conn, DbPool};
use crate::{
    models::comment::comment_saved::{CommentSaved, CommentSavedForm},
    traits::Saveable,
};
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;

impl CommentSaved {
    /// Takes a list of comment ids, and for each post, return whether it's saved or not
    pub async fn get_saved_for_ids(
        pool: &DbPool,
        ids: Vec<i32>,
        for_user_id: i32,
    ) -> Result<Vec<(i32, bool)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{comment_saved, comments};

        comments::table
            .left_join(
                comment_saved::table.on(comment_saved::comment_id
                    .eq(comments::id)
                    .and(comment_saved::user_id.eq(for_user_id))),
            )
            .filter(comments::id.eq_any(ids))
            .select((comments::id, comment_saved::id.nullable()))
            .load::<(i32, Option<i32>)>(conn)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(comment_id_, save_id)| (comment_id_, save_id.is_some()))
                    .collect::<Vec<(i32, bool)>>()
            })
    }
}

#[async_trait::async_trait]
impl Saveable for CommentSaved {
    type Form = CommentSavedForm;

    async fn save(pool: &DbPool, form: &CommentSavedForm) -> Result<Self, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::insert_into(comment_saved)
            .values(form)
            .on_conflict((comment_id, user_id))
            .do_update()
            .set(form)
            .get_result::<Self>(conn)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not save comment"))
    }

    async fn unsave(pool: &DbPool, form: &CommentSavedForm) -> Result<usize, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(
            comment_saved
                .filter(comment_id.eq(form.comment_id))
                .filter(user_id.eq(form.user_id)),
        )
        .execute(conn)
        .await
        .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not unsave comment"))
    }
}
