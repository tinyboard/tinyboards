use crate::models::post::post_report::PostReport;
use crate::{
    models::moderator::mod_actions::{
        ModLockPost, ModLockPostForm, ModRemovePost, ModRemovePostForm,
    },
    models::post::posts::{Post, PostForm},
    schema::{posts, post_report},
    traits::{Crud, Moderateable},
    utils::{naive_now, get_conn, DbPool, FETCH_LIMIT_MAX},
    newtypes::DbUrl,
};
use diesel::{prelude::*, result::Error};
use tinyboards_utils::TinyBoardsError;
use diesel_async::RunQueryDsl;
use url::Url;

impl Post {

    pub async fn list_for_board(pool: &DbPool, the_board_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        posts::table
            .filter(posts::board_id.eq(the_board_id))
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false))
            .then_order_by(posts::featured_board.desc())
            .then_order_by(posts::creation_date.desc())
            .limit(FETCH_LIMIT_MAX)
            .load::<Self>(conn)
            .await
    }

    pub async fn list_featured_for_board(pool: &DbPool, the_board_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        posts::table
            .filter(posts::board_id.eq(the_board_id))
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false))
            .filter(posts::featured_board.eq(true))
            .then_order_by(posts::creation_date.desc())
            .limit(FETCH_LIMIT_MAX)
            .load::<Self>(conn)
            .await
    }

    pub async fn read_from_apub_id(pool: &DbPool, object_id: Url) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        let object_id: DbUrl = object_id.into();
        Ok(
            posts::table
                .filter(posts::ap_id.eq(object_id))
                .first::<Post>(conn)
                .await
                .ok()
                .map(Into::into)
        )
    }

    pub async fn resolve_reports(pool: &DbPool, post_id: i32, resolver_id: i32) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(post_report::table.filter(post_report::post_id.eq(post_id)))
            .set((post_report::resolved.eq(true),
                post_report::resolver_id.eq(resolver_id)))
            .get_results::<PostReport>(conn)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not resolve reports"))
    }

    pub async fn permadelete_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
      ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        let perma_deleted = "*Permananently Deleted*";
        let perma_deleted_url = "https://deleted.com";
    
        diesel::update(posts.filter(creator_id.eq(for_creator_id)))
          .set((
            title.eq(perma_deleted),
            url.eq(perma_deleted_url),
            body.eq(perma_deleted),
            is_deleted.eq(true),
            updated.eq(naive_now()),
          ))
          .get_results::<Self>(conn)
          .await
    }

    pub async fn submit(pool: &DbPool, form: PostForm) -> Result<Self, TinyBoardsError> {
        Self::create(pool, &form)
            .await    
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not submit posts"))
    }

    pub fn is_post_creator(person_id: i32, post_creator_id: i32) -> bool {
        person_id == post_creator_id
    }

    pub async fn fetch_image_posts_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        let image_search = "%/media%";
        posts
            .filter(creator_id.eq(for_creator_id))
            .filter(url.like(image_search))
            .load::<Self>(conn)
            .await
    }

    pub async fn fetch_image_posts_for_board(
        pool: &DbPool,
        for_board_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        let image_search = "%/media%";
        posts
            .filter(board_id.eq(for_board_id))
            .filter(url.like(image_search))
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
        let image_search = "%/media%";

        diesel::update(
            posts
                .filter(creator_id.eq(for_creator_id))
                .filter(url.like(image_search)),
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
        let image_search = "%/media%";

        diesel::update(
            posts
                .filter(board_id.eq(for_board_id))
                .filter(url.like(image_search)),
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

    pub async fn update_featured_board(
        pool: &DbPool,
        post_id: i32,
        new_featured: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((featured_board.eq(new_featured), updated.eq(naive_now())))
            .get_result::<Self>(conn)
            .await
    }

    pub async fn update_featured_local(
        pool: &DbPool,
        post_id: i32,
        new_featured: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(post_id))
            .set((featured_local.eq(new_featured), updated.eq(naive_now())))
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

    pub async fn update_removed_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
        for_board_id: Option<i32>,
        new_removed: bool,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::posts::dsl::*;
        let conn = &mut get_conn(pool).await?;
        let mut update = diesel::update(posts).into_boxed();
        update = update.filter(creator_id.eq(for_creator_id));
        if let Some(for_board_id) = for_board_id {
            update = update.filter(board_id.eq(for_board_id));
        }

        update
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .get_results::<Self>(conn)
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
            .on_conflict(posts::ap_id)
            .do_update()
            .set(form)
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
            mod_person_id: admin_id.unwrap_or(1),
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
            mod_person_id: admin_id.unwrap_or(1),
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
            mod_person_id: admin_id.unwrap_or(1),
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
            mod_person_id: admin_id.unwrap_or(1),
            post_id: self.id,
            locked: Some(Some(false)),
        };

        ModLockPost::create(pool, &lock_form).await.map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }
}
