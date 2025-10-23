/**
 * Registration Application Queries for Admins
 */
use crate::{DbPool, LoggedInUser};
use async_graphql::*;
use tinyboards_db::models::user::user::User;
use tinyboards_db::models::site::registration_applications::RegistrationApplication;
use tinyboards_db::traits::Crud;
use tinyboards_utils::TinyBoardsError;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Default)]
pub struct RegistrationApplicationQueries;

#[derive(SimpleObject)]
pub struct RegistrationApplicationView {
    pub id: i32,
    pub user_id: i32,
    pub username: String,
    pub answer: String,
    pub admin_id: Option<i32>,
    pub deny_reason: Option<String>,
    pub created_at: String,
}

#[Object]
impl RegistrationApplicationQueries {
    /// List all pending registration applications (admin only)
    pub async fn list_registration_applications(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<RegistrationApplicationView>> {
        let user = ctx.data_unchecked::<LoggedInUser>().inner();
        let pool = ctx.data::<DbPool>()?;

        // Check if user is admin
        if let Some(user) = user {
            if user.admin_level == 0 {
                return Err(TinyBoardsError::from_message(403, "Not an admin").into());
            }
        } else {
            return Err(TinyBoardsError::from_message(401, "Not logged in").into());
        }

        let conn = &mut tinyboards_db::utils::get_conn(pool).await?;
        
        use tinyboards_db::schema::{registration_applications, users};
        
        let mut query = registration_applications::table
            .inner_join(users::table.on(registration_applications::user_id.eq(users::id)))
            .select((
                registration_applications::id,
                registration_applications::user_id,
                users::name,
                registration_applications::answer,
                registration_applications::admin_id,
                registration_applications::deny_reason,
                registration_applications::creation_date,
            ))
            .into_boxed();

        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i64);
        } else {
            query = query.limit(50); // Default limit
        }

        if let Some(offset_val) = offset {
            query = query.offset(offset_val as i64);
        }

        let applications: Vec<(i32, i32, String, String, Option<i32>, Option<String>, chrono::NaiveDateTime)> = 
            query.load(conn).await?;

        let result = applications
            .into_iter()
            .map(|(id, user_id, username, answer, admin_id, deny_reason, creation_date)| {
                RegistrationApplicationView {
                    id,
                    user_id,
                    username,
                    answer,
                    admin_id,
                    deny_reason,
                    created_at: creation_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a specific registration application by ID (admin only)
    pub async fn get_registration_application(
        &self,
        ctx: &Context<'_>,
        application_id: i32,
    ) -> Result<Option<RegistrationApplicationView>> {
        let user = ctx.data_unchecked::<LoggedInUser>().inner();
        let pool = ctx.data::<DbPool>()?;

        // Check if user is admin
        if let Some(user) = user {
            if user.admin_level == 0 {
                return Err(TinyBoardsError::from_message(403, "Not an admin").into());
            }
        } else {
            return Err(TinyBoardsError::from_message(401, "Not logged in").into());
        }

        let application = match RegistrationApplication::read(pool, application_id).await {
            Ok(app) => app,
            Err(_) => return Ok(None),
        };

        let user = User::read(pool, application.user_id).await?;

        Ok(Some(RegistrationApplicationView {
            id: application.id,
            user_id: application.user_id,
            username: user.name,
            answer: application.answer,
            admin_id: application.admin_id,
            deny_reason: application.deny_reason,
            created_at: application.creation_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        }))
    }

    /// Get count of pending registration applications (admin only)
    pub async fn registration_applications_count(&self, ctx: &Context<'_>) -> Result<i32> {
        let user = ctx.data_unchecked::<LoggedInUser>().inner();
        let pool = ctx.data::<DbPool>()?;

        // Check if user is admin
        if let Some(user) = user {
            if user.admin_level == 0 {
                return Err(TinyBoardsError::from_message(403, "Not an admin").into());
            }
        } else {
            return Err(TinyBoardsError::from_message(401, "Not logged in").into());
        }

        let conn = &mut tinyboards_db::utils::get_conn(pool).await?;
        
        use tinyboards_db::schema::registration_applications;
        use diesel::dsl::count;
        
        let count: i64 = registration_applications::table
            .select(count(registration_applications::id))
            .first(conn)
            .await?;

        Ok(count as i32)
    }
}