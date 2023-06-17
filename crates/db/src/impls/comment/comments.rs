use crate::newtypes::DbUrl;
use crate::schema::comments::dsl::*;
use crate::traits::Moderateable;
use crate::utils::naive_now;
use crate::{
    models::comment::comments::{Comment, CommentForm},
    models::moderator::mod_actions::{ModRemoveComment, ModRemoveCommentForm},
    traits::Crud,
    utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error, QueryDsl};
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;
use url::Url;

impl Comment {

    pub async fn read_from_apub_id(pool: &DbPool, object_id: Url) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        let object_id: DbUrl = object_id.into();
        Ok(
            comments
                .filter(ap_id.eq(object_id))
                .first::<Comment>(conn)
                .await
                .ok()
                .map(Into::into),
        )
    }

    pub fn parent_comment_id(&self) -> Option<i32> {
        let parent_comment_id = self.parent_id;
        parent_comment_id
    }

    pub async fn submit(pool: &DbPool, form: CommentForm) -> Result<Self, TinyBoardsError> {
        Self::create(pool, &form)
            .await    
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not submit comment"))
    }
    /// Checks if a comment with a given id exists. Don't use if you need a whole Comment object.
    pub async fn check_if_exists(
        pool: &DbPool,
        cid: i32,
    ) -> Result<Option<i32>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        comments
            .select(id)
            .filter(id.eq(cid))
            .first::<i32>(conn)
            .await
            .optional()
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "error while checking existence of comment",
                )
            })
    }

    pub fn is_comment_creator(person_id: i32, comment_creator_id: i32) -> bool {
        person_id == comment_creator_id
    }

    pub async fn update_deleted(
        pool: &DbPool,
        comment_id: i32,
        new_deleted: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(comment_id))
            .set((is_deleted.eq(new_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_removed(
        pool: &DbPool,
        comment_id: i32,
        new_removed: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(comment_id))
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_removed_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
        new_removed: bool,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        diesel::update(comments.filter(creator_id.eq(for_creator_id)))
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .get_results::<Self>(conn)
            .await
    }

    pub async fn update_locked(
        pool: &DbPool,
        comment_id: i32,
        locked: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(comment_id))
            .set((is_locked.eq(locked), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn get_by_id(pool: &DbPool, cid: i32) -> Result<Option<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        comments
            .filter(id.eq(cid))
            .first::<Self>(conn)
            .await
            .optional()
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not get comment by id"))
    }

    /// Loads list of comments replying to the specified post.
    pub async fn replies_to_post(
        pool: &DbPool,
        pid: i32,
    ) -> Result<Vec<Self>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        comments
            .filter(post_id.eq(pid))
            .load::<Self>(conn)
            .await
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "could not get replies to post")
            })
    }
}

#[async_trait::async_trait]
impl Crud for Comment {
    type Form = CommentForm;
    type IdType = i32;

    async fn read(pool: &DbPool, comment_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        comments.find(comment_id).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, comment_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(comments.find(comment_id)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &CommentForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_comment = diesel::insert_into(comments)
            .values(form)
            .get_result::<Self>(conn)
            .await?;
        Ok(new_comment)
    }
    async fn update(pool: &DbPool, comment_id: i32, form: &CommentForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(comments.find(comment_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Moderateable for Comment {
    fn get_board_id(&self) -> i32 {
        self.board_id
    }

    async fn remove(
        &self,
        admin_id: Option<i32>,
        reason: Option<String>,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError> {
        Self::update_removed(pool, self.id, true)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove comment"))?;

        // create mod log entry
        let remove_comment_form = ModRemoveCommentForm {
            mod_person_id: admin_id.unwrap_or(1),
            comment_id: self.id,
            reason: Some(reason),
            removed: Some(Some(true)),
        };

        ModRemoveComment::create(pool, &remove_comment_form)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to remove comment"))?;

        Ok(())
    }

    async fn approve(
        &self,
        admin_id: Option<i32>,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError> {
        Self::update_removed(pool, self.id, false)
            .await
            .map(|_| ())
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to approve comment")
            })?;

        // create mod log entry
        let remove_comment_form = ModRemoveCommentForm {
            mod_person_id: admin_id.unwrap_or(1),
            comment_id: self.id,
            reason: None,
            removed: Some(Some(false)),
        };

        ModRemoveComment::create(pool, &remove_comment_form).await.map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Failed to approve comment")
        })?;

        Ok(())
    }

    async fn lock(&self, _admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError> {
        Self::update_locked(pool, self.id, true)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // TODO: modlog entry for comment lock

        Ok(())
    }

    async fn unlock(
        &self,
        _admin_id: Option<i32>,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError> {
        Self::update_locked(pool, self.id, false)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // TODO: modlog entry for comment unlock

        Ok(())
    }
}
