use crate::aggregates::structs::{CommentAggregates, PersonAggregates};
use crate::models::comment::comment_report::CommentReport;
use crate::models::person::person::Person;
use crate::newtypes::DbUrl;
use crate::schema::comments::dsl::*;
use crate::traits::Moderateable;
use crate::utils::functions::hot_rank;
use crate::utils::{fuzzy_search, limit_and_offset_unlimited, naive_now};
use crate::{
    models::comment::comments::{Comment, CommentForm},
    models::moderator::mod_actions::{ModRemoveComment, ModRemoveCommentForm},
    schema::comment_report,
    traits::Crud,
    utils::{get_conn, DbPool},
};
use crate::{CommentSortType, ListingType};
use diesel::{prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use tinyboards_utils::TinyBoardsError;
use url::Url;

impl Comment {
    /// Returns a list of users who commented on a given post.
    pub async fn load_participants_for_post(
        pool: &DbPool,
        for_post_id: i32,
    ) -> Result<Vec<(Person, PersonAggregates)>, Error> {
        use crate::schema::{comments, person, person_aggregates};
        let conn = &mut get_conn(pool).await?;

        comments::table
            .inner_join(person::table)
            .inner_join(
                person_aggregates::table.on(person_aggregates::person_id.eq(comments::creator_id)),
            )
            .filter(comments::post_id.eq(for_post_id))
            .select((person::all_columns, person_aggregates::all_columns))
            .distinct_on(comments::creator_id)
            .load::<(Person, PersonAggregates)>(conn)
            .await
    }

    /// List comments with counts
    pub async fn load_with_counts(
        pool: &DbPool,
        person_id_join: i32,
        sort: CommentSortType,
        listing_type: ListingType,
        page: Option<i64>,
        limit: Option<i64>,
        for_user: Option<i32>,
        for_post: Option<i32>,
        for_board: Option<i32>,
        saved_only: bool,
        search_term: Option<String>,
        include_deleted: bool,
        include_removed: bool,
        include_banned_boards: bool,
    ) -> Result<Vec<(Self, CommentAggregates)>, Error> {
        use crate::schema::{
            board_mods, board_subscriber, boards, comment_aggregates, comment_saved, comments,
        };
        let conn = &mut get_conn(pool).await?;

        let mut query = comments::table
            .inner_join(comment_aggregates::table)
            .inner_join(boards::table.on(boards::id.eq(comments::board_id)))
            .left_join(
                board_mods::table.on(board_mods::board_id
                    .eq(comments::board_id)
                    .and(board_mods::person_id.eq(person_id_join))),
            )
            .left_join(
                board_subscriber::table.on(board_subscriber::board_id
                    .eq(comments::board_id)
                    .and(board_subscriber::person_id.eq(person_id_join))),
            )
            .left_join(
                comment_saved::table.on(comment_saved::comment_id
                    .eq(comments::id)
                    .and(comment_saved::person_id.eq(person_id_join))),
            )
            .select((comments::all_columns, comment_aggregates::all_columns))
            .into_boxed();

        if !include_deleted {
            query = query.filter(comments::is_deleted.eq(false));
        }

        if !include_removed {
            query = query.filter(comments::is_removed.eq(false));
        }

        if saved_only {
            query = query.filter(comment_saved::id.is_not_null());
        }

        if let Some(person_id) = for_user {
            query = query.filter(comments::creator_id.eq(person_id));
        }

        if let Some(board_id_) = for_board {
            query = query.filter(comments::board_id.eq(board_id_));
        }

        if let Some(post_id_) = for_post {
            query = query.filter(comments::post_id.eq(post_id_));
        }

        if let Some(search) = search_term {
            query = query.filter(comments::body.ilike(fuzzy_search(search.as_str())));
        }

        query = match listing_type {
            ListingType::All => query,
            ListingType::Subscribed => query.filter(board_subscriber::id.is_not_null()),
            ListingType::Local => query.filter(comments::local.eq(true)),
            ListingType::Moderated => query.filter(board_mods::id.is_not_null()),
        };

        query = match sort {
            CommentSortType::Hot => query
                .then_order_by(
                    hot_rank(comment_aggregates::score, comment_aggregates::creation_date).desc(),
                )
                .then_order_by(comment_aggregates::creation_date.desc()),
            CommentSortType::New => query.then_order_by(comments::creation_date.desc()),
            CommentSortType::Old => query.then_order_by(comments::creation_date.asc()),
            CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
        };

        let (limit, offset) = limit_and_offset_unlimited(page, limit);

        query = query.limit(limit).offset(offset);

        query.load::<(Self, CommentAggregates)>(conn).await
    }

    pub async fn permadelete_for_creator(
        pool: &DbPool,
        for_creator_id: i32,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::comments::dsl::*;
        let conn = &mut get_conn(pool).await?;
        diesel::update(comments.filter(creator_id.eq(for_creator_id)))
            .set((
                body.eq("[ contents deleted permanently ]"),
                body_html.eq("<p>[ contents deleted permanently ]</p>"),
                is_deleted.eq(true),
                updated.eq(naive_now()),
            ))
            .get_results::<Self>(conn)
            .await
    }

    pub async fn read_from_apub_id(pool: &DbPool, object_id: Url) -> Result<Option<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        let object_id: DbUrl = object_id.into();
        Ok(comments
            .filter(ap_id.eq(object_id))
            .first::<Comment>(conn)
            .await
            .ok()
            .map(Into::into))
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
    pub async fn check_if_exists(pool: &DbPool, cid: i32) -> Result<Option<i32>, TinyBoardsError> {
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

    pub async fn set_removed(&self, pool: &DbPool, new_removed: bool) -> Result<(), Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;
        diesel::update(comments.find(self.id))
            .set((is_removed.eq(new_removed), updated.eq(naive_now())))
            .execute(conn)
            .await
            .map(|_| ())
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
    pub async fn replies_to_post(pool: &DbPool, pid: i32) -> Result<Vec<Self>, TinyBoardsError> {
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

    pub async fn resolve_reports(
        pool: &DbPool,
        comment_id: i32,
        resolver_id: i32,
    ) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(comment_report::table.filter(comment_report::comment_id.eq(comment_id)))
            .set((
                comment_report::resolved.eq(true),
                comment_report::resolver_id.eq(resolver_id),
            ))
            .get_results::<CommentReport>(conn)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "could not resolve reports"))
    }
}

#[async_trait::async_trait]
impl Crud for Comment {
    type Form = CommentForm;
    type IdType = i32;

    async fn read(pool: &DbPool, comment_id: i32) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        comments.find(comment_id).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, comment_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(comments.find(comment_id))
            .execute(conn)
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

    async fn approve(&self, admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError> {
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

        ModRemoveComment::create(pool, &remove_comment_form)
            .await
            .map_err(|e| {
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

    async fn unlock(&self, _admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError> {
        Self::update_locked(pool, self.id, false)
            .await
            .map(|_| ())
            .map_err(|e| TinyBoardsError::from(e))?;

        // TODO: modlog entry for comment unlock

        Ok(())
    }
}
