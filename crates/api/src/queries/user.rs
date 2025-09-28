use crate::helpers::validation::check_private_instance;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::{
    models::{
        user::{user::User as DbUser, user_subscriber::UserSubscriber, user::UserSettings as DbUserSettings},
        post::{posts::Post as DbPost, post_saved::PostSaved},
        comment::{comments::Comment as DbComment, comment_saved::CommentSaved},
    },
    traits::Crud,
    utils::DbPool,
    SortType as DbSortType,
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::{user::{User, UserSettings}, post::Post, comment::Comment}, UserListingType, UserSortType, SortType};

#[derive(Default)]
pub struct QueryUser;

#[Object]
impl QueryUser {
    pub async fn user(&self, context: &Context<'_>, name: String) -> Result<User> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        if name.contains("@") {
            return Err(TinyBoardsError::from_message(501, "Federation not supported").into());
        }

        let db_user = DbUser::get_by_name(pool, name)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "User not found."))?;

        // Fetch user aggregates for proper stats
        use tinyboards_db::aggregates::structs::UserAggregates;
        let user_aggregates = UserAggregates::read(pool, db_user.id)
            .await
            .unwrap_or_else(|_| UserAggregates {
                id: 0,
                user_id: db_user.id,
                post_count: 0,
                post_score: 0,
                comment_count: 0,
                comment_score: 0,
            });

        Ok(User::from((db_user, user_aggregates)))
    }


    pub async fn list_users(
        &self,
        context: &Context<'_>,
        search_term: Option<String>,
        listing_type: Option<UserListingType>,
        sort: Option<UserSortType>,
        page: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<User>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(UserSortType::MostRep);
        let listing_type = listing_type.unwrap_or(UserListingType::NotBanned);

        let users = DbUser::list_with_counts(
            pool,
            sort.into(),
            limit,
            page,
            listing_type.into(),
            search_term,
        )
        .await
        .map_err(|e| {
            TinyBoardsError::from_error_message(e, 500, "Server error while fetching users.")
        })?;

        Ok(users.into_iter().map(|(user, aggregates)| User::from((user, aggregates))).collect::<Vec<User>>())
    }

    /// Get list of followers for a user
    pub async fn user_followers(
        &self,
        context: &Context<'_>,
        user_id: i32,
    ) -> Result<Vec<User>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let followers = UserSubscriber::get_followers(pool, user_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get followers"))?;

        // Convert UserSubscriber records to User records
        let mut result = Vec::new();
        for subscriber in followers {
            // For followers, we want the subscriber (follower) user data
            if let Ok(user_data) = DbUser::read(pool, subscriber.subscriber_id).await {
                let default_aggregates = tinyboards_db::aggregates::structs::UserAggregates {
                    id: 0,
                    user_id: user_data.id,
                    post_count: 0,
                    post_score: 0,
                    comment_count: 0,
                    comment_score: 0,
                };
                result.push(User::from((user_data, default_aggregates)));
            }
        }
        Ok(result)
    }

    /// Get list of users that a user is following
    pub async fn user_following(
        &self,
        context: &Context<'_>,
        user_id: i32,
    ) -> Result<Vec<User>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let following = UserSubscriber::get_following(pool, user_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get following list"))?;

        // Convert UserSubscriber records to User records
        let mut result = Vec::new();
        for subscription in following {
            // For following, we want the user being followed (user_id)
            if let Ok(user_data) = DbUser::read(pool, subscription.user_id).await {
                let default_aggregates = tinyboards_db::aggregates::structs::UserAggregates {
                    id: 0,
                    user_id: user_data.id,
                    post_count: 0,
                    post_score: 0,
                    comment_count: 0,
                    comment_score: 0,
                };
                result.push(User::from((user_data, default_aggregates)));
            }
        }
        Ok(result)
    }

    /// Get pending follow requests for the current user
    pub async fn pending_follow_requests(
        &self,
        context: &Context<'_>,
    ) -> Result<Vec<User>> {
        let pool = context.data::<DbPool>()?;
        let user = context.data::<LoggedInUser>()?.require_user_not_banned()?;

        let pending_requests = UserSubscriber::get_pending_requests(pool, user.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get pending follow requests"))?;

        // Convert UserSubscriber records to User records
        let mut result = Vec::new();
        for request in pending_requests {
            // For pending requests, we want the requester (subscriber) user data
            if let Ok(user_data) = DbUser::read(pool, request.subscriber_id).await {
                let default_aggregates = tinyboards_db::aggregates::structs::UserAggregates {
                    id: 0,
                    user_id: user_data.id,
                    post_count: 0,
                    post_score: 0,
                    comment_count: 0,
                    comment_score: 0,
                };
                result.push(User::from((user_data, default_aggregates)));
            }
        }
        Ok(result)
    }

    /// Check if current user is following another user
    pub async fn is_following_user(
        &self,
        context: &Context<'_>,
        user_id: i32,
    ) -> Result<bool> {
        let pool = context.data::<DbPool>()?;
        let user = context.data::<LoggedInUser>()?.require_user_not_banned()?;

        let is_following = UserSubscriber::is_following(pool, user.id, user_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to check following status"))?;

        Ok(is_following)
    }

    /// Get current user's private settings
    pub async fn get_user_settings(&self, context: &Context<'_>) -> Result<UserSettings> {
        let pool = context.data::<DbPool>()?;
        let user = context.data::<LoggedInUser>()?.require_user_not_banned()?;

        // Use the existing UserSettings struct that includes private fields
        let db_user_settings = DbUserSettings {
            id: user.id,
            name: user.name.clone(),
            email: user.email.clone(),
            show_nsfw: user.show_nsfw,
            show_bots: user.show_bots,
            theme: user.theme.clone(),
            default_sort_type: user.default_sort_type,
            default_listing_type: user.default_listing_type,
            email_notifications_enabled: user.email_notifications_enabled,
            interface_language: user.interface_language.clone(),
            updated: user.updated,
        };

        Ok(UserSettings::from(db_user_settings))
    }

    /// Get saved posts for authenticated user with pagination
    pub async fn get_user_saved_posts(
        &self,
        context: &Context<'_>,
        limit: Option<i64>,
        page: Option<i64>,
        sort: Option<SortType>,
    ) -> Result<Vec<Post>> {
        let pool = context.data::<DbPool>()?;
        let user = context.data::<LoggedInUser>()?.require_user_not_banned()?;

        let limit = limit.unwrap_or(10).min(50); // Default 10, max 50
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        let sort_type = sort.unwrap_or(SortType::New);

        // Get saved post IDs with pagination
        use tinyboards_db::schema::{post_saved, posts, post_aggregates};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::utils::get_conn;

        let conn = &mut get_conn(pool).await?;

        let saved_posts_query = post_saved::table
            .inner_join(posts::table.on(post_saved::post_id.eq(posts::id)))
            .left_join(post_aggregates::table.on(posts::id.eq(post_aggregates::post_id)))
            .filter(post_saved::user_id.eq(user.id))
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false))
            .limit(limit)
            .offset(offset);

        let saved_posts: Vec<(tinyboards_db::models::post::post_saved::PostSaved, DbPost, Option<tinyboards_db::aggregates::structs::PostAggregates>)> = match sort_type {
            SortType::New => saved_posts_query
                .order_by(post_saved::creation_date.desc())
                .load(conn)
                .await?,
            SortType::Hot => saved_posts_query
                .order_by(post_aggregates::hot_rank.desc().nulls_last())
                .load(conn)
                .await?,
            SortType::TopDay | SortType::TopWeek | SortType::TopMonth | SortType::TopYear | SortType::TopAll =>
                saved_posts_query
                    .order_by(post_aggregates::score.desc().nulls_last())
                    .load(conn)
                    .await?,
            _ => saved_posts_query
                .order_by(post_saved::creation_date.desc())
                .load(conn)
                .await?,
        };

        let posts: Vec<Post> = saved_posts
            .into_iter()
            .map(|(_, post, aggregates)| {
                let default_aggregates = tinyboards_db::aggregates::structs::PostAggregates {
                    id: 0,
                    post_id: post.id,
                    comments: 0,
                    score: 0,
                    upvotes: 0,
                    downvotes: 0,
                    creation_date: post.creation_date,
                    newest_comment_time: post.creation_date,
                    newest_comment_time_necro: Some(post.creation_date),
                    featured_board: false,
                    featured_local: false,
                    hot_rank: 0,
                    hot_rank_active: 0,
                    board_id: post.board_id,
                    creator_id: post.creator_id,
                    controversy_rank: 0.0,
                };
                Post::from((post, aggregates.unwrap_or(default_aggregates)))
            })
            .collect();

        Ok(posts)
    }

    /// Get user's post history with pagination
    pub async fn get_user_post_history(
        &self,
        context: &Context<'_>,
        username: String,
        limit: Option<i64>,
        page: Option<i64>,
        sort: Option<SortType>,
    ) -> Result<Vec<Post>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let target_user = DbUser::get_by_name(pool, username)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "User not found"))?;

        let limit = limit.unwrap_or(10).min(50); // Default 10, max 50
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        let sort_type = sort.unwrap_or(SortType::New);

        use tinyboards_db::schema::{posts, post_aggregates};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::utils::get_conn;

        let conn = &mut get_conn(pool).await?;

        let posts_query = posts::table
            .left_join(post_aggregates::table.on(posts::id.eq(post_aggregates::post_id)))
            .filter(posts::creator_id.eq(target_user.id))
            .filter(posts::is_deleted.eq(false))
            .filter(posts::is_removed.eq(false))
            .limit(limit)
            .offset(offset);

        let user_posts: Vec<(DbPost, Option<tinyboards_db::aggregates::structs::PostAggregates>)> = match sort_type {
            SortType::New => posts_query
                .order_by(posts::creation_date.desc())
                .load(conn)
                .await?,
            SortType::Hot => posts_query
                .order_by(post_aggregates::hot_rank.desc().nulls_last())
                .load(conn)
                .await?,
            SortType::TopDay | SortType::TopWeek | SortType::TopMonth | SortType::TopYear | SortType::TopAll =>
                posts_query
                    .order_by(post_aggregates::score.desc().nulls_last())
                    .load(conn)
                    .await?,
            _ => posts_query
                .order_by(posts::creation_date.desc())
                .load(conn)
                .await?,
        };

        let posts: Vec<Post> = user_posts
            .into_iter()
            .map(|(post, aggregates)| {
                let default_aggregates = tinyboards_db::aggregates::structs::PostAggregates {
                    id: 0,
                    post_id: post.id,
                    comments: 0,
                    score: 0,
                    upvotes: 0,
                    downvotes: 0,
                    creation_date: post.creation_date,
                    newest_comment_time: post.creation_date,
                    newest_comment_time_necro: Some(post.creation_date),
                    featured_board: false,
                    featured_local: false,
                    hot_rank: 0,
                    hot_rank_active: 0,
                    board_id: post.board_id,
                    creator_id: post.creator_id,
                    controversy_rank: 0.0,
                };
                Post::from((post, aggregates.unwrap_or(default_aggregates)))
            })
            .collect();

        Ok(posts)
    }

    /// Get user's comment history with pagination
    pub async fn get_user_comment_history(
        &self,
        context: &Context<'_>,
        username: String,
        limit: Option<i64>,
        page: Option<i64>,
        sort: Option<SortType>,
    ) -> Result<Vec<Comment>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let target_user = DbUser::get_by_name(pool, username)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "User not found"))?;

        let limit = limit.unwrap_or(10).min(50); // Default 10, max 50
        let page = page.unwrap_or(1).max(1);
        let offset = (page - 1) * limit;
        let sort_type = sort.unwrap_or(SortType::New);

        use tinyboards_db::schema::{comments, comment_aggregates};
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use tinyboards_db::utils::get_conn;

        let conn = &mut get_conn(pool).await?;

        let comments_query = comments::table
            .left_join(comment_aggregates::table.on(comments::id.eq(comment_aggregates::comment_id)))
            .filter(comments::creator_id.eq(target_user.id))
            .filter(comments::is_deleted.eq(false))
            .filter(comments::is_removed.eq(false))
            .limit(limit)
            .offset(offset);

        let user_comments: Vec<(DbComment, Option<tinyboards_db::aggregates::structs::CommentAggregates>)> = match sort_type {
            SortType::New => comments_query
                .order_by(comments::creation_date.desc())
                .load(conn)
                .await?,
            SortType::Hot => comments_query
                .order_by(comment_aggregates::hot_rank.desc().nulls_last())
                .load(conn)
                .await?,
            SortType::TopDay | SortType::TopWeek | SortType::TopMonth | SortType::TopYear | SortType::TopAll =>
                comments_query
                    .order_by(comment_aggregates::score.desc().nulls_last())
                    .load(conn)
                    .await?,
            _ => comments_query
                .order_by(comments::creation_date.desc())
                .load(conn)
                .await?,
        };

        let comments: Vec<Comment> = user_comments
            .into_iter()
            .map(|(comment, aggregates)| {
                let default_aggregates = tinyboards_db::aggregates::structs::CommentAggregates {
                    id: 0,
                    comment_id: comment.id,
                    score: 0,
                    upvotes: 0,
                    downvotes: 0,
                    creation_date: comment.creation_date,
                    child_count: 0,
                    hot_rank: 0,
                    controversy_rank: 0.0,
                };
                Comment::from((comment, aggregates.unwrap_or(default_aggregates)))
            })
            .collect();

        Ok(comments)
    }
}