use crate::{
    models::moderator::mod_actions::{
        ModLockPost, ModLockPostForm, ModRemovePost, ModRemovePostForm,
    },
    models::post::posts::{Post, PostForm},
    schema::posts,
    traits::{Crud, Moderateable},
    utils::naive_now,
};
use diesel::{prelude::*, result::Error, PgConnection};
use tinyboards_utils::TinyBoardsError;

impl Post {
    pub fn submit(conn: &mut PgConnection, form: PostForm) -> Result<Self, TinyBoardsError> {
        Self::create(conn, &form)
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not submit posts"))
    }

    pub fn is_post_creator(user_id: i32, post_creator_id: i32) -> bool {
        user_id == post_creator_id
    }

    pub fn fetch_image_posts_for_creator(
        conn: &mut PgConnection,
        for_creator_id: i32,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::posts::dsl::*;
        let pictrs_search = "%/image%";
        posts
            .filter(creator_id.eq(for_creator_id))
            .filter(url.like(pictrs_search))
            .load::<Self>(conn)
    }

    pub fn fetch_image_posts_for_board(
        conn: &mut PgConnection,
        for_board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::posts::dsl::*;
        let pictrs_search = "%/image%";
        posts
            .filter(board_id.eq(for_board_id))
            .filter(url.like(pictrs_search))
            .load::<Self>(conn)
    }

    /// Sets the url and thumbnails fields to None
    pub fn remove_post_images_and_thumbnails_for_creator(
        conn: &mut PgConnection,
        for_creator_id: i32,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::posts::dsl::*;
        let pictrs_search = "%/image%";

        diesel::update(
            posts
                .filter(creator_id.eq(for_creator_id))
                .filter(url.like(pictrs_search)),
        )
        .set((
            url.eq::<Option<String>>(None),
            thumbnail_url.eq::<Option<String>>(None),
        ))
        .get_results::<Self>(conn)
    }

    /// Sets the url and thumbnails fields to None
    pub fn remove_post_images_and_thumbnails_for_board(
        conn: &mut PgConnection,
        for_board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::posts::dsl::*;
        let pictrs_search = "%/image%";

        diesel::update(
            posts
                .filter(board_id.eq(for_board_id))
                .filter(url.like(pictrs_search)),
        )
        .set((
            url.eq::<Option<String>>(None),
            thumbnail_url.eq::<Option<String>>(None),
        ))
        .get_results::<Self>(conn)
    }

    /// Checks if a posts with a given id exists. Don't use if you need a whole posts object.
    pub fn check_if_exists(
        conn: &mut PgConnection,
        pid: i32,
    ) -> Result<Option<i32>, TinyBoardsError> {
        use crate::schema::posts::dsl::*;
        posts
            .select(id)
            .filter(id.eq(pid))
            .first::<i32>(conn)
            .optional()
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "error while checking existence of posts",
                )
            })
    }

    pub fn update_locked(
        conn: &mut PgConnection,
        post_id: i32,
        new_locked: bool,
    ) -> Result<Self, Error> {
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_locked.eq(new_locked), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_stickied(
        conn: &mut PgConnection,
        post_id: i32,
        new_stickied: bool,
    ) -> Result<Self, Error> {
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_stickied.eq(new_stickied), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_deleted(
        conn: &mut PgConnection,
        post_id: i32,
        new_deleted: bool,
    ) -> Result<Self, Error> {
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_deleted.eq(new_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }

    pub fn update_removed(
        conn: &mut PgConnection,
        post_id: i32,
        new_removed: bool,
    ) -> Result<Self, Error> {
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .get_result::<Self>(conn)
    }
}

impl Crud for Post {
    type Form = PostForm;
    type IdType = i32;

    fn read(conn: &mut PgConnection, post_id: i32) -> Result<Self, Error> {
        posts::table.find(post_id).first::<Self>(conn)
    }
    fn delete(conn: &mut PgConnection, post_id: i32) -> Result<usize, Error> {
        diesel::delete(posts::table.find(post_id)).execute(conn)
    }
    fn create(conn: &mut PgConnection, form: &PostForm) -> Result<Self, Error> {
        let new_post = diesel::insert_into(posts::table)
            .values(form)
            .get_result::<Self>(conn)?;

        Ok(new_post)
    }
    fn update(conn: &mut PgConnection, post_id: i32, form: &PostForm) -> Result<Self, Error> {
        diesel::update(posts::table.find(post_id))
            .set(form)
            .get_result::<Self>(conn)
    }
}

impl Moderateable for Post {
    fn get_board_id(&self) -> i32 {
        self.board_id
    }

    fn remove(
        &self,
        admin_id: Option<i32>,
        reason: Option<String>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError> {
        Self::update_removed(conn, self.id, true)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting remove action to mod log
        let remove_post_form = ModRemovePostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            reason: Some(reason),
            removed: Some(Some(true)),
        };

        // submit mod action to mod log
        ModRemovePost::create(conn, &remove_post_form).map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    fn approve(
        &self,
        admin_id: Option<i32>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError> {
        Self::update_removed(conn, self.id, false)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting remove action to mod log
        let remove_post_form = ModRemovePostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            reason: None,
            removed: Some(Some(false)),
        };

        // submit mod action to mod log
        ModRemovePost::create(conn, &remove_post_form).map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    fn lock(&self, admin_id: Option<i32>, conn: &mut PgConnection) -> Result<(), TinyBoardsError> {
        Self::update_locked(conn, self.id, true)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting lock action for mod log
        let lock_form = ModLockPostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            locked: Some(Some(true)),
        };

        ModLockPost::create(conn, &lock_form).map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    fn unlock(
        &self,
        admin_id: Option<i32>,
        conn: &mut PgConnection,
    ) -> Result<(), TinyBoardsError> {
        Self::update_locked(conn, self.id, false)
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting lock action for mod log
        let lock_form = ModLockPostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            locked: Some(Some(false)),
        };

        ModLockPost::create(conn, &lock_form).map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }
}
