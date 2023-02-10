use crate::{
    models::moderator::mod_actions::{
        ModLockPost, ModLockPostForm, ModRemovePost, ModRemovePostForm,
    },
    models::post::posts::{Post, PostForm},
    schema::posts,
    traits::{Crud, Moderateable},
    utils::{naive_now, get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;

impl Post {
    pub async fn submit(pool: &DbPool, form: PostForm) -> Result<Self, TinyBoardsError> {
        Self::create(pool, &form)
            .await    
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not submit posts"))
    }

    pub fn is_post_creator(user_id: i32, post_creator_id: i32) -> bool {
        user_id == post_creator_id
    }

    pub async fn fetch_image_posts_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        let pictrs_search = "%/image%";
        posts
            .filter(creator_id.eq(for_creator_id))
            .filter(url.like(pictrs_search))
            .load::<Self>(conn)
            .await
    }

    pub async fn fetch_image_posts_for_board(
        pool: &DbPool,
        for_board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        let pictrs_search = "%/image%";
        posts
            .filter(board_id.eq(for_board_id))
            .filter(url.like(pictrs_search))
            .load::<Self>(conn)
            .await
    }

    /// Sets the url and thumbnails fields to None
    pub async fn remove_post_images_and_thumbnails_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
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
        .await
    }

    /// Sets the url and thumbnails fields to None
    pub async fn remove_post_images_and_thumbnails_for_board(
        pool: &DbPool,
        for_board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
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
        .await
    }

    /// Checks if a posts with a given id exists. Don't use if you need a whole posts object.
    pub async fn check_if_exists(
        pool: &DbPool,
        pid: i32,
    ) -> Result<Option<i32>, TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        posts
            .select(id)
            .filter(id.eq(pid))
            .first::<i32>(conn)
            .await
            .optional()
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "error while checking existence of posts",
                )
            })
    }

    pub async fn update_locked(
        pool: &DbPool,
        post_id: i32,
        new_locked: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_locked.eq(new_locked), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_stickied(
        pool: &DbPool,
        post_id: i32,
        new_stickied: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_stickied.eq(new_stickied), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_deleted(
        pool: &DbPool,
        post_id: i32,
        new_deleted: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_deleted.eq(new_deleted), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_removed(
        pool: &DbPool,
        post_id: i32,
        new_removed: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Crud for Post {
    type Form = PostForm;
    type IdType = i32;

    async fn read(pool: &DbPool, post_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        posts::table.find(post_id).first::<Self>(conn)
        .await
    }
    async fn delete(pool: &DbPool, post_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(posts::table.find(post_id)).execute(conn)
        .await
    }
    async fn create(pool: &DbPool, form: &PostForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        let new_post = diesel::insert_into(posts::table)
            .values(form)
            .get_result::<Self>(conn)
            .await?;

        Ok(new_post)
    }
    async fn update(pool: &DbPool, post_id: i32, form: &PostForm) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(posts::table.find(post_id))
            .set(form)
            .get_result::<Self>(conn)
            .await
    }
}

#[async_trait::async_trait]
impl Moderateable for Post {
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
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting remove action to mod log
        let remove_post_form = ModRemovePostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            reason: Some(reason),
            removed: Some(Some(true)),
        };

        // submit mod action to mod log
        ModRemovePost::create(pool, &remove_post_form).await.map_err(|e| TinyBoardsError::from(e))?;

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
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting remove action to mod log
        let remove_post_form = ModRemovePostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            reason: None,
            removed: Some(Some(false)),
        };

        // submit mod action to mod log
        ModRemovePost::create(pool, &remove_post_form).await.map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    async fn lock(&self, admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError> {
        Self::update_locked(pool, self.id, true)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting lock action for mod log
        let lock_form = ModLockPostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            locked: Some(Some(true)),
        };

        ModLockPost::create(pool, &lock_form).await.map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    async fn unlock(
        &self,
        admin_id: Option<i32>,
        pool: &DbPool,
    ) -> Result<(), TinyBoardsError> {
        Self::update_locked(pool, self.id, false)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // form for submitting lock action for mod log
        let lock_form = ModLockPostForm {
            mod_user_id: admin_id.unwrap_or(1),
            post_id: self.id,
            locked: Some(Some(false)),
        };

        ModLockPost::create(pool, &lock_form).await.map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }
}
