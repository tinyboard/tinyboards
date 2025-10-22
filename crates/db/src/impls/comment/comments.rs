use crate::aggregates::structs::CommentAggregates;
use crate::models::comment::comment_report::CommentReport;
use crate::models::user::User;
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

impl Comment {
    pub async fn list_comments_for_board(pool: &DbPool, the_board_id: i32) -> Result<Vec<Self>, Error> {
        use crate::schema::posts;
        let conn = &mut get_conn(pool).await?;
        comments
            .inner_join(posts::table.on(posts::id.eq(post_id)))
            .filter(posts::board_id.eq(the_board_id))
            .select(comments::all_columns())
            .load::<Self>(conn)
            .await
    }

    pub async fn list_all_comments(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        comments
            .load::<Self>(conn)
            .await
    }

    pub async fn update_body_html(pool: &DbPool, comment_id: i32, new_body_html: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(comments.find(comment_id))
            .set(body_html.eq(new_body_html))
            .get_result::<Self>(conn)
            .await
    }

    /// Returns a list of the 5 most recent unique users who commented on a given post.
    pub async fn load_participants_for_post(
        pool: &DbPool,
        for_post_id: i32,
    ) -> Result<Vec<User>, Error> {
        use crate::schema::{comments, users};
        let conn = &mut get_conn(pool).await?;

        // Get all comments for this post ordered by most recent first
        let all_comments: Vec<Comment> = comments::table
            .filter(comments::post_id.eq(for_post_id))
            .order(comments::creation_date.desc())
            .load::<Comment>(conn)
            .await?;

        // Get unique creator_ids in order of first appearance (most recent)
        let mut seen = std::collections::HashSet::new();
        let unique_creator_ids: Vec<i32> = all_comments
            .iter()
            .filter_map(|c| {
                if seen.insert(c.creator_id) {
                    Some(c.creator_id)
                } else {
                    None
                }
            })
            .take(5)
            .collect();

        // Fetch the users
        if unique_creator_ids.is_empty() {
            return Ok(vec![]);
        }

        let users_map: std::collections::HashMap<i32, User> = users::table
            .filter(users::id.eq_any(&unique_creator_ids))
            .load::<User>(conn)
            .await?
            .into_iter()
            .map(|user| (user.id, user))
            .collect();

        // Return users in the same order as unique_creator_ids (most recent first)
        Ok(unique_creator_ids
            .into_iter()
            .filter_map(|user_id| users_map.get(&user_id).cloned())
            .collect())
    }

    /// Get a single comment with its counts
    pub async fn get_with_counts(
        pool: &DbPool,
        id_: i32,
    ) -> Result<(Self, CommentAggregates), Error> {
        use crate::schema::{comment_aggregates, comments};
        let conn = &mut get_conn(pool).await?;

        comments::table
            .inner_join(comment_aggregates::table)
            .filter(comments::id.eq(id_))
            .select((comments::all_columns, comment_aggregates::all_columns))
            .first::<(Self, CommentAggregates)>(conn)
            .await
    }

    /// List comments with counts
    pub async fn load_with_counts(
        pool: &DbPool,
        user_id_join: i32,
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
        parent_ids: Option<&Vec<i32>>,
        max_depth: Option<i32>,
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
                    .and(board_mods::user_id.eq(user_id_join))),
            )
            .left_join(
                board_subscriber::table.on(board_subscriber::board_id
                    .eq(comments::board_id)
                    .and(board_subscriber::user_id.eq(user_id_join))),
            )
            .left_join(
                comment_saved::table.on(comment_saved::comment_id
                    .eq(comments::id)
                    .and(comment_saved::user_id.eq(user_id_join))),
            )
            .select((comments::all_columns, comment_aggregates::all_columns))
            .into_boxed();

        if let Some(max_depth) = max_depth {
            // Depth limit
            query = query.filter(comments::level.le(max_depth));
        }

        if !include_banned_boards {
            query = query.filter(
                boards::is_removed
                    .eq(false)
                    .and(boards::is_deleted.eq(false)),
            );
        }

        if !include_deleted {
            query = query.filter(comments::is_deleted.eq(false));
        }

        if !include_removed {
            query = query.filter(comments::is_removed.eq(false));
        }

        if saved_only {
            query = query.filter(comment_saved::id.is_not_null());
        }

        if let Some(user_id) = for_user {
            query = query.filter(comments::creator_id.eq(user_id));
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

        if let Some(parent_ids) = parent_ids {
            query = query.filter(comments::parent_id.eq_any(parent_ids));
        }

        query = match listing_type {
            ListingType::All => query,
            ListingType::Subscribed => query.filter(board_subscriber::id.is_not_null()),
            ListingType::Local => query,
            ListingType::Moderated => query.filter(board_mods::id.is_not_null()),
        };

        query = match sort {
            CommentSortType::Hot => query
                .then_order_by(comments::is_pinned.desc())
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

    /// List a comment with its replies and optionally parents (max value for context is 3) (returns a tuple with the top comment id and the comments)
    pub async fn get_with_replies_and_counts(
        pool: &DbPool,
        id_: i32,
        context: Option<u16>,
        sort: CommentSortType,
        user_id_join: i32,
        check_for_post_id: Option<i32>,
        max_depth: Option<i32>,
    ) -> Result<(i32, Vec<(Self, CommentAggregates)>), Error> {
        //let conn = &mut get_conn(pool).await?;

        let context = std::cmp::min(context.unwrap_or(0), 3) as i32;
        let comment_with_counts = Self::get_with_counts(pool, id_).await?;
        let max_depth = max_depth.unwrap_or(6);

        // Check if the comment belongs to a given post
        if let Some(post_id_) = check_for_post_id {
            if post_id_ != comment_with_counts.0.post_id {
                return Err(Error::NotFound);
            }
        }

        let mut parent_id_opt = comment_with_counts.0.parent_id;
        let mut ids = vec![comment_with_counts.0.id];
        let mut comment_list = vec![comment_with_counts];
        let mut top_comment_id = id_;

        // load parent comment, and then the parent of the parent comment, ...
        for _ in 0..context {
            if let Some(parent_id_) = parent_id_opt {
                let parent_comment_with_counts = Self::get_with_counts(pool, parent_id_).await?;
                top_comment_id = parent_comment_with_counts.0.id;
                parent_id_opt = parent_comment_with_counts.0.parent_id;
                comment_list.push(parent_comment_with_counts);
            }
        }

        // load comment replies, and then the replies of those comments, ...
        for _ in 0..max_depth - context {
            let mut replies_with_counts = Self::load_with_counts(
                pool,
                user_id_join,
                sort,
                ListingType::All,
                None,
                None,
                None,
                None,
                None,
                false,
                None,
                true,
                true,
                true,
                Some(&ids),
                None,
            )
            .await?;
            ids = replies_with_counts
                .iter()
                .map(|(comment, _)| comment.id)
                .collect::<Vec<i32>>();

            comment_list.append(&mut replies_with_counts);
        }

        Ok((top_comment_id, comment_list))
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

    pub fn is_comment_creator(user_id: i32, comment_creator_id: i32) -> bool {
        user_id == comment_creator_id
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

    pub async fn update_pinned(
        pool: &DbPool,
        comment_id: i32,
        on_post: i32,
        new_pinned: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::comments::dsl::*;

        // There can be only one pinned comment. When pinning a comment, unpin the comment that's already pinned under that post.
        if new_pinned {
            diesel::update(comments.filter(post_id.eq(on_post)))
                .set(is_pinned.eq(false))
                .get_result::<Self>(conn)
                .await?;
        }

        diesel::update(comments.find(comment_id))
            .set((is_pinned.eq(new_pinned), updated.eq(naive_now())))
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

    /// Load comments chronologically for thread posts (no tree structure)
    pub async fn load_chronological_for_post(
        pool: &DbPool,
        for_post_id: i32,
        limit: Option<i64>,
        page: Option<i64>,
        include_deleted: bool,
        include_removed: bool,
    ) -> Result<Vec<(Self, CommentAggregates)>, Error> {
        use crate::schema::{comment_aggregates, comments};
        let conn = &mut get_conn(pool).await?;

        let (limit, offset) = limit_and_offset_unlimited(page, limit);

        let mut query = comments::table
            .inner_join(comment_aggregates::table)
            .filter(comments::post_id.eq(for_post_id))
            .into_boxed();

        if !include_deleted {
            query = query.filter(comments::is_deleted.eq(false));
        }

        if !include_removed {
            query = query.filter(comments::is_removed.eq(false));
        }

        // Sort chronologically by creation_date (ignore parent_id/level)
        query
            .order_by(comments::creation_date.asc())
            .limit(limit)
            .offset(offset)
            .select((comments::all_columns, comment_aggregates::all_columns))
            .load::<(Self, CommentAggregates)>(conn)
            .await
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
            mod_user_id: admin_id.unwrap_or(1),
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
            mod_user_id: admin_id.unwrap_or(1),
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
