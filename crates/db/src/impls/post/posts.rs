use crate::aggregates::structs::PostAggregates;
use crate::models::post::post_report::PostReport;
//use crate::pagination::Paginate;
use crate::utils::functions::hot_rank;
use crate::utils::limit_and_offset;
use crate::{
    models::moderator::mod_actions::{
        ModLockPost, ModLockPostForm, ModRemovePost, ModRemovePostForm,
    },
    models::post::posts::{Post, PostForm},
    schema::{post_report, posts},
    traits::{Crud, Moderateable},
    utils::{get_conn, naive_now, DbPool, FETCH_LIMIT_MAX},
};
use crate::{ListingType, SortType};
use diesel::dsl::{now, IntervalDsl};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;
use regex::Regex;
use tinyboards_utils::TinyBoardsError;

impl Post {
    pub async fn list_posts_for_board(pool: &DbPool, the_board_id: i32) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        posts::table
            .filter(posts::board_id.eq(the_board_id))
            .load::<Self>(conn)
            .await
    }

    pub async fn list_all_posts(pool: &DbPool) -> Result<Vec<Self>, Error> {
        let conn = &mut get_conn(pool).await?;
        posts::table
            .load::<Self>(conn)
            .await
    }

    pub async fn update_body_html(pool: &DbPool, post_id: i32, new_body_html: &str) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::update(posts::table.find(post_id))
            .set(posts::body_html.eq(new_body_html))
            .get_result::<Self>(conn)
            .await
    }

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

    pub async fn list_featured_for_board(
        pool: &DbPool,
        the_board_id: i32,
    ) -> Result<Vec<Self>, Error> {
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


    pub async fn resolve_reports(
        pool: &DbPool,
        post_id: i32,
        resolver_id: i32,
    ) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(post_report::table.filter(post_report::post_id.eq(post_id)))
            .set((
                post_report::resolved.eq(true),
                post_report::resolver_id.eq(resolver_id),
            ))
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

    pub async fn set_removed(&self, pool: &DbPool, value: bool) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(self.id))
            .set(is_removed.eq(value))
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to set post removed value.")
            })
    }

    pub async fn set_locked(&self, pool: &DbPool, value: bool) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(self.id))
            .set(is_locked.eq(value))
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| {
                TinyBoardsError::from_error_message(e, 500, "Failed to set post locked value.")
            })
    }

    pub async fn set_featured_board(
        &self,
        pool: &DbPool,
        value: bool,
    ) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(self.id))
            .set(featured_board.eq(value))
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "Failed to set post featured board value.",
                )
            })
    }

    pub async fn set_featured_local(
        &self,
        pool: &DbPool,
        value: bool,
    ) -> Result<(), TinyBoardsError> {
        let conn = &mut get_conn(pool).await?;

        use crate::schema::posts::dsl::*;
        diesel::update(posts.find(self.id))
            .set(featured_local.eq(value))
            .execute(conn)
            .await
            .map(|_| ())
            .map_err(|e| {
                TinyBoardsError::from_error_message(
                    e,
                    500,
                    "Failed to set post featured local value.",
                )
            })
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

    /// Takes the title and generates a chunk for use in the post's URL.
    pub fn generate_chunk(title: String) -> String {
        let title = &title.split(" ").collect::<Vec<&str>>();
        let slice_max_index = if title.len() >= 7 { 7 } else { title.len() };
        let mut title = title[0..slice_max_index].join("-").to_lowercase();

        // these are all I could think of at the moment, feel free to expand this
        let replaces = [
            ["á", "a"],
            ["ä", "ae"],
            ["é", "e"],
            ["ó", "o"],
            ["ú", "u"],
            ["ö", "oe"],
            ["ü", "ue"],
            ["ő", "o"],
            ["ű", "u"],
            ["ß", "ss"],
            [" ", "-"],
        ];

        for [from, to] in replaces.iter() {
            title = str::replace(&title, from, to);
        }

        let chunk_regex = Regex::new(r"[^a-zA-Z0-9\-]").unwrap();
        let chunk = chunk_regex.replace_all(&title, "").to_string();

        if chunk.is_empty() {
            "-".to_string()
        } else {
            chunk
        }
    }

    /// Checks if a posts with a given id exists. Don't use if you need a whole posts object.
    pub async fn check_if_exists(pool: &DbPool, pid: i32) -> Result<Option<i32>, TinyBoardsError> {
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

    /// Load a single post with its associated aggregates table.
    pub async fn get_with_counts(
        pool: &DbPool,
        id: i32,
        require_board_not_banned: bool,
    ) -> Result<(Self, PostAggregates), Error> {
        use crate::schema::{boards, post_aggregates, posts};
        let conn = &mut get_conn(pool).await?;

        let mut query = posts::table
            .inner_join(post_aggregates::table)
            .inner_join(boards::table)
            .filter(posts::id.eq(id))
            .select((posts::all_columns, post_aggregates::all_columns))
            .into_boxed();

        if require_board_not_banned {
            query = query.filter(
                boards::is_removed
                    .eq(false)
                    .and(boards::is_deleted.eq(false)),
            );
        }

        query.first::<(Self, PostAggregates)>(conn).await
    }

    /// Load posts for the given ids.
    pub async fn load_with_counts_for_ids(
        pool: &DbPool,
        ids: Vec<i32>,
        require_board_not_banned: bool,
    ) -> Result<Vec<(Self, PostAggregates)>, Error> {
        use crate::schema::{boards, post_aggregates, posts};
        let conn = &mut get_conn(pool).await?;

        let mut query = posts::table
            .inner_join(post_aggregates::table)
            .inner_join(boards::table)
            .filter(posts::id.eq_any(ids))
            .select((posts::all_columns, post_aggregates::all_columns))
            .into_boxed();

        if require_board_not_banned {
            query = query.filter(
                boards::is_removed
                    .eq(false)
                    .and(boards::is_deleted.eq(false)),
            );
        }

        query.load::<(Self, PostAggregates)>(conn).await
    }

    /// Load posts which match the specified criteria, with their associated aggregate tables.
    /// To be used for graphql queries. When you need all data, use `PostView`.
    pub async fn load_with_counts(
        pool: &DbPool,
        user_id_join: i32,
        limit: Option<i64>,
        page: Option<i64>,
        show_deleted: bool,
        show_removed: bool,
        include_banned_boards: bool,
        saved_only: bool,
        board_id: Option<i32>,
        user_id: Option<i32>,
        sort: SortType,
        listing_type: ListingType,
    ) -> Result<Vec<(Self, PostAggregates)>, Error> {
        use crate::schema::{
            board_mods, board_subscriber, boards, post_aggregates, post_saved, posts,
        };
        let conn = &mut get_conn(pool).await?;

        let mut query = posts::table
            .inner_join(boards::table)
            .inner_join(post_aggregates::table)
            .left_join(
                post_saved::table.on(post_saved::post_id
                    .eq(posts::id)
                    .and(post_saved::user_id.eq(user_id_join))),
            )
            .left_join(
                board_mods::table.on(board_mods::board_id
                    .eq(posts::board_id)
                    .and(board_mods::user_id.eq(user_id_join))),
            )
            .left_join(
                board_subscriber::table.on(board_subscriber::board_id
                    .eq(posts::board_id)
                    .and(board_subscriber::user_id.eq(user_id_join))),
            )
            .select((posts::all_columns, post_aggregates::all_columns))
            .into_boxed();

        if saved_only {
            query = query.filter(post_saved::id.is_not_null());
        }

        if !show_removed {
            query = query.filter(posts::is_removed.eq(false));
        }

        if !show_deleted {
            query = query.filter(posts::is_deleted.eq(false));
        }

        if !include_banned_boards {
            query = query.filter(
                boards::is_removed
                    .eq(false)
                    .and(boards::is_deleted.eq(false)),
            );
        }

        if let Some(user_id) = user_id {
            query = query.filter(posts::creator_id.eq(user_id));
        }

        if let Some(board_id) = board_id {
            query = query.filter(posts::board_id.eq(board_id));
        }

        // Exclude thread posts from feed listings (only show feed posts)
        // Thread posts are only visible in the /b/[board]/threads section
        query = query.filter(posts::post_type.eq("feed"));

        match listing_type {
            // All posts feed: hide posts from hidden boards and boards excluded from /all, except those which the user is a member of
            ListingType::All => {
                query = query.filter(
                    boards::is_hidden
                        .eq(false)
                        .and(boards::exclude_from_all.eq(false))
                        .or(board_subscriber::id.is_not_null()),
                )
            }
            // Subscribed boards: home page, hide nothing
            ListingType::Subscribed => query = query.filter(board_subscriber::id.is_not_null()),
            // Local: all posts \ hidden boards
            ListingType::Local => {
                query = query.filter(
                    boards::is_hidden
                        .eq(false)
                        .or(board_subscriber::id.is_not_null()),
                )
            }
            // Mod feed: only posts that the user moderates
            ListingType::Moderated => query = query.filter(board_mods::id.is_not_null()),
        };

        query = match sort {
            SortType::Active => query
                .then_order_by(
                    hot_rank(post_aggregates::score, post_aggregates::newest_comment_time).desc(),
                )
                .then_order_by(post_aggregates::newest_comment_time.desc()),
            SortType::Hot => query
                .then_order_by(
                    hot_rank(post_aggregates::score, post_aggregates::creation_date).desc(),
                )
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::New => query.then_order_by(post_aggregates::creation_date.desc()),
            SortType::Old => query.then_order_by(post_aggregates::creation_date.asc()),
            SortType::NewComments => {
                query.then_order_by(post_aggregates::newest_comment_time.desc())
            }
            SortType::MostComments => query
                .then_order_by(post_aggregates::comments.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopAll => query
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopYear => query
                .filter(post_aggregates::creation_date.gt(now - 1.years()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopMonth => query
                .filter(post_aggregates::creation_date.gt(now - 1.months()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopWeek => query
                .filter(post_aggregates::creation_date.gt(now - 1.weeks()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::TopDay => query
                .filter(post_aggregates::creation_date.gt(now - 1.days()))
                .then_order_by(post_aggregates::score.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
            SortType::Controversial => query
                .then_order_by(post_aggregates::controversy_rank.desc())
                .then_order_by(post_aggregates::creation_date.desc()),
        };

        let (limit, offset) = limit_and_offset(page, limit)?;

        query = query.limit(limit).offset(offset);

        query.load::<(Self, PostAggregates)>(conn).await
    }

    /// List thread posts for a board (sorted by activity)
    pub async fn list_threads_for_board(
        pool: &DbPool,
        the_board_id: i32,
        limit: Option<i64>,
        page: Option<i64>,
    ) -> Result<Vec<Self>, Error> {
        use crate::schema::{post_aggregates, posts};
        let conn = &mut get_conn(pool).await?;

        let (limit, offset) = limit_and_offset(page, limit)?;

        posts::table
            .inner_join(post_aggregates::table)
            .filter(posts::board_id.eq(the_board_id))
            .filter(posts::post_type.eq("thread"))
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false))
            .order_by(posts::featured_board.desc()) // Pinned first
            .then_order_by(post_aggregates::newest_comment_time.desc()) // Then by activity
            .limit(limit)
            .offset(offset)
            .select(posts::all_columns)
            .load::<Self>(conn)
            .await
    }

    /// Pin or unpin a thread post
    pub async fn pin_thread(
        pool: &DbPool,
        post_id: i32,
        pinned: bool,
    ) -> Result<Self, Error> {
        let conn = &mut get_conn(pool).await?;

        diesel::update(posts::table.find(post_id))
            .set((
                posts::featured_board.eq(pinned),
                posts::updated.eq(Some(naive_now())),
            ))
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
        posts::table.find(post_id).first::<Self>(conn).await
    }
    async fn delete(pool: &DbPool, post_id: i32) -> Result<usize, Error> {
        let conn = &mut get_conn(pool).await?;
        diesel::delete(posts::table.find(post_id))
            .execute(conn)
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

impl Post {
    /// Load posts for a stream feed based on flair and board subscriptions
    pub async fn load_stream_feed(
        pool: &DbPool,
        _user_id: i32,
        flair_ids: &[i32],
        board_ids: &[i32],
        sort_type: SortType,
        show_nsfw: bool,
        max_posts_per_board: Option<i32>,
        post_type_filter: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Self, PostAggregates)>, Error> {
        let conn = &mut get_conn(pool).await?;
        use crate::schema::{post_aggregates, posts, post_flairs};

        // Build base query
        let mut query = posts::table
            .inner_join(post_aggregates::table)
            .left_join(post_flairs::table.on(post_flairs::post_id.eq(posts::id)))
            .into_boxed();

        // Filter by flair IDs or board IDs
        if !flair_ids.is_empty() && !board_ids.is_empty() {
            query = query.filter(
                post_flairs::flair_template_id.eq_any(flair_ids)
                    .or(posts::board_id.eq_any(board_ids))
            );
        } else if !flair_ids.is_empty() {
            query = query.filter(post_flairs::flair_template_id.eq_any(flair_ids));
        } else if !board_ids.is_empty() {
            query = query.filter(posts::board_id.eq_any(board_ids));
        } else {
            // No subscriptions, return empty
            return Ok(Vec::new());
        }

        // Filter out deleted and removed posts
        query = query
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false));

        // NSFW filter
        if !show_nsfw {
            query = query.filter(posts::is_nsfw.eq(false));
        }

        // Filter by post type if specified (feed or thread)
        if let Some(post_type) = post_type_filter {
            query = query.filter(posts::post_type.eq(post_type));
        }

        // Apply sorting
        query = match sort_type {
            SortType::Hot => query.order_by(hot_rank(post_aggregates::score, post_aggregates::creation_date).desc()),
            SortType::Active => query.order_by(hot_rank(post_aggregates::score, post_aggregates::newest_comment_time).desc()),
            SortType::New => query.order_by(posts::creation_date.desc()),
            SortType::Old => query.order_by(posts::creation_date.asc()),
            SortType::TopDay => query
                .filter(posts::creation_date.gt(now - 1.days()))
                .order_by(post_aggregates::score.desc()),
            SortType::TopWeek => query
                .filter(posts::creation_date.gt(now - 1.weeks()))
                .order_by(post_aggregates::score.desc()),
            SortType::TopMonth => query
                .filter(posts::creation_date.gt(now - 1.months()))
                .order_by(post_aggregates::score.desc()),
            SortType::TopYear => query
                .filter(posts::creation_date.gt(now - 1.years()))
                .order_by(post_aggregates::score.desc()),
            SortType::TopAll => query.order_by(post_aggregates::score.desc()),
            SortType::MostComments => query.order_by(post_aggregates::comments.desc()),
            _ => query.order_by(posts::creation_date.desc()),
        };

        // Apply limit and offset
        query = query.limit(limit).offset(offset);

        // Execute query
        let results = query
            .select((posts::all_columns, post_aggregates::all_columns))
            .load::<(Self, PostAggregates)>(conn)
            .await?;

        // Apply max_posts_per_board limit if specified
        if let Some(max_per_board) = max_posts_per_board {
            let mut board_counts: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
            let filtered_results: Vec<(Self, PostAggregates)> = results
                .into_iter()
                .filter(|(post, _)| {
                    let count = board_counts.entry(post.board_id).or_insert(0);
                    if *count < max_per_board {
                        *count += 1;
                        true
                    } else {
                        false
                    }
                })
                .collect();

            Ok(filtered_results)
        } else {
            Ok(results)
        }
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
        ModRemovePost::create(pool, &remove_post_form)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    async fn approve(&self, admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError> {
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
        ModRemovePost::create(pool, &remove_post_form)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

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

        ModLockPost::create(pool, &lock_form)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }

    async fn unlock(&self, admin_id: Option<i32>, pool: &DbPool) -> Result<(), TinyBoardsError> {
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

        ModLockPost::create(pool, &lock_form)
            .await
            .map_err(|e| TinyBoardsError::from(e))?;

        Ok(())
    }
}

