use crate::helpers::validation::check_private_instance;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::{
    models::person::{person::Person as DbPerson, person_subscriber::PersonSubscriber, user::User as DbUser}, 
    utils::DbPool
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::person::Person, UserListingType, UserSortType};

#[derive(Default)]
pub struct QueryPerson;

#[Object]
impl QueryPerson {
    pub async fn user(&self, context: &Context<'_>, name: String) -> Result<Person> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        if name.contains("@") {
            todo!("Add apub support here");
        }

        let db_person = DbPerson::get_user_for_name(pool, name)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "User not found."))?;

        Ok(Person::from(db_person))
    }

    /// Alias for `user`.
    pub async fn person(&self, context: &Context<'_>, name: String) -> Result<Person> {
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
    ) -> Result<Vec<Person>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(UserSortType::MostRep);
        let listing_type = listing_type.unwrap_or(UserListingType::NotBanned);

        let users = DbPerson::list_with_counts(
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

        Ok(users.into_iter().map(Person::from).collect::<Vec<Person>>())
    }

    /// Get list of followers for a user
    pub async fn user_followers(
        &self,
        context: &Context<'_>,
        person_id: i32,
    ) -> Result<Vec<Person>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let followers = PersonSubscriber::list_subscribers(pool, person_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get followers"))?;

        Ok(followers.into_iter().map(Person::from).collect::<Vec<Person>>())
    }

    /// Get list of users that a person is following
    pub async fn user_following(
        &self,
        context: &Context<'_>,
        person_id: i32,
    ) -> Result<Vec<Person>> {
        let pool = context.data::<DbPool>()?;
        let v_opt = context.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let following = PersonSubscriber::list_following(pool, person_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get following list"))?;

        Ok(following.into_iter().map(Person::from).collect::<Vec<Person>>())
    }

    /// Get pending follow requests for the current user
    pub async fn pending_follow_requests(
        &self,
        context: &Context<'_>,
    ) -> Result<Vec<Person>> {
        let pool = context.data::<DbPool>()?;
        let user = context.data::<LoggedInUser>()?.require_user_not_banned()?;

        let pending_requests = PersonSubscriber::list_pending_follow_requests(pool, user.person.id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to get pending follow requests"))?;

        Ok(pending_requests.into_iter().map(Person::from).collect::<Vec<Person>>())
    }

    /// Check if current user is following another user
    pub async fn is_following_user(
        &self,
        context: &Context<'_>,
        person_id: i32,
    ) -> Result<bool> {
        let pool = context.data::<DbPool>()?;
        let user = context.data::<LoggedInUser>()?.require_user_not_banned()?;

        let is_following = PersonSubscriber::is_following(pool, user.person.id, person_id)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to check following status"))?;

        Ok(is_following)
    }
}
