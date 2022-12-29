use crate::schema::comments::dsl::*;
use crate::traits::Moderateable;
use crate::utils::naive_now;
use crate::{
    models::comment::comments::{Comment, CommentForm},
    models::moderator::mod_actions::{ModRemoveComment, ModRemoveCommentForm},
    traits::Crud,
};
use diesel::{prelude::*, result::Error, PgConnection, QueryDsl, RunQueryDsl};
use tinyboards_utils::TinyBoardsError;

impl Comment {
    pub fn submit(conn: &mut PgConnection, form: CommentForm) -> Result<Self, TinyBoardsError> {
        Self::create(conn, &form)
            .map_err(|e| TinyBoardsError::from_error_message(e, "could not submit comment"))
    }
    /// Checks if a comment with a given id exists. Don't use if you need a whole Comment object.
    pub fn check_if_exists(
        conn: &mut PgConnection,
        cid: i32,
    ) -> Result<Option<i32>, TinyBoardsError> {
        use crate::schema::comments::dsl::*;
        comments
            .select(id)
            .filter(id.eq(cid))
            .first::<i32>(conn)
            .optional()
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, "error while checking existence of comment")
            })
    }

    pub fn is_comment_creator(user_id: i32, comment_creator_id: i32) -> bool {
        user_id == comment_creator_id
    }

    pub fn update_deleted(
        conn: &mut PgConnection,
        comment_id: i32,
        new_deleted: bool,
    ) -> Result<Self, Error> {
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(comment_id))
            .set((is_deleted.eq(new_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_removed(
        conn: &mut PgConnection,
        comment_id: i32,
        new_removed: bool,
    ) -> Result<Self, Error> {
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(comment_id))
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_locked(
        conn: &mut PgConnection,
        comment_id: i32,
        locked: bool,
    ) -> Result<Self, Error> {
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(comment_id))
            .set((is_locked.eq(locked), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn get_by_id(conn: &mut PgConnection, cid: i32) -> Result<Option<Self>, TinyBoardsError> {
        use crate::schema::comments::dsl::*;
        comments
            .filter(id.eq(cid))
            .first::<Self>(conn)
            .optional()
            .map_err(|e| TinyBoardsError::from_error_message(e, "could not get comment by id"))
    }

    /// Loads list of comments replying to the specified post.
    pub fn replies_to_post(
        conn: &mut PgConnection,
        pid: i32,
    ) -> Result<Vec<Self>, TinyBoardsError> {
        use crate::schema::comments::dsl::*;
        comments
            .filter(post_id.eq(pid))
            .load::<Self>(conn)
            .map_err(|e| TinyBoardsError::from_error_message(e, "could not get replies to post"))
    }
}

impl Crud for Comment {
    type Form = CommentForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, comment_id: i32) -> Result<Self, Error> {
        comments.find(comment_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, comment_id: i32) -> Result<usize, Error> {
        diesel::delete(comments.find(comment_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &CommentForm) -> Result<Self, Error> {
        let new_comment = diesel::insert_into(comments)
            .values(form)
            .get_result::<Self>(conn)?;
        Ok(new_comment)
    }
    fn update(conn: &mut PgConnection, comment_id: i32, form: &CommentForm) -> Result<Self, Error> {
        diesel::update(comments.find(comment_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Moderateable for Comment {
    fn remove(
        &self,
        admin_id: Option<i32>,
        reason: Option<String>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError> {
        Self::update_removed(conn, self.id, true)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from_error_message(e, "Failed to remove comment"))?;

        // create mod log entry
        let remove_comment_form = ModRemoveCommentForm {
            mod_user_id: admin_id.unwrap_or(1),
            comment_id: self.id,
            reason: Some(reason),
            removed: Some(Some(true)),
        };

        ModRemoveComment::create(conn, &remove_comment_form)
            .map_err(|e| TinyBoardsError::from_error_message(e, "Failed to remove comment"))?;

        Ok(())
    }

    fn approve(
        &self,
        admin_id: Option<i32>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError> {
        Self::update_removed(conn, self.id, false)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from_error_message(e, "Failed to approve comment"))?;

        // create mod log entry
        let remove_comment_form = ModRemoveCommentForm {
            mod_user_id: admin_id.unwrap_or(1),
            comment_id: self.id,
            reason: None,
            removed: Some(Some(false)),
        };

        ModRemoveComment::create(conn, &remove_comment_form)
            .map_err(|e| TinyBoardsError::from_error_message(e, "Failed to approve comment"))?;

        Ok(())
    }

    fn lock(&self, _admin_id: Option<i32>, conn: &mut PgConnection) -> Result<(), TinyBoardsError> {
        Self::update_locked(conn, self.id, true)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // TODO: modlog entry for comment lock

        Ok(())
    }

    fn unlock(
        &self,
        _admin_id: Option<i32>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError> {
        Self::update_locked(conn, self.id, false)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // TODO: modlog entry for comment unlock

        Ok(())
    }
}
