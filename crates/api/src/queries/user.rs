use crate::helpers::validation::check_private_instance;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::{
    models::user::{user::User as DbUser, user_subscriber::UserSubscriber},
    traits::Crud,
    utils::DbPool
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::user::User, UserListingType, UserSortType};

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

        Ok(User::from(db_user))
    }

    /// Alias for `user`.
    pub async fn person(&self, context: &Context<'_>, name: String) -> Result<User> {
        self.user(context, name).await
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

        Ok(users.into_iter().map(User::from).collect::<Vec<User>>())
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
}