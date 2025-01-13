use crate::helpers::validation::check_private_instance;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::{models::person::person::Person as DbPerson, utils::DbPool};
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
}
