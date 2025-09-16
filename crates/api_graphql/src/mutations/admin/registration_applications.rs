/**
 * Registration Application Management for Admins
 */
use crate::{DbPool, LoggedInUser};
use async_graphql::*;
use tinyboards_db::models::user::user::{User, UserForm};
use tinyboards_db::models::site::registration_applications::RegistrationApplication;
use tinyboards_db::traits::Crud;
use tinyboards_utils::TinyBoardsError;

#[derive(Default)]
pub struct RegistrationApplicationMutations;

#[Object]
impl RegistrationApplicationMutations {
    /// Approve a registration application
    pub async fn approve_registration_application(
        &self,
        ctx: &Context<'_>,
        application_id: i32,
    ) -> Result<bool> {
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

        // Get the registration application
        let application = RegistrationApplication::read(pool, application_id).await?;

        // Update the user to mark application as accepted
        let user_form = UserForm {
            is_application_accepted: Some(true),
            accepted_application: Some(true),
            ..UserForm::default()
        };

        // Find the user by user_id
        let user = User::read(pool, application.user_id).await?;
        User::update(pool, user.id, &user_form).await?;

        // Delete the application since it's been processed
        RegistrationApplication::delete(pool, application_id).await?;

        Ok(true)
    }

    /// Deny a registration application
    pub async fn deny_registration_application(
        &self,
        ctx: &Context<'_>,
        application_id: i32,
        _reason: Option<String>,
    ) -> Result<bool> {
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

        // Get the registration application
        let application = RegistrationApplication::read(pool, application_id).await?;

        // Store the denial reason before deletion
        if _reason.is_some() {
            // You could optionally update the application with denial reason
            // before deletion for logging purposes
        }

        // Delete the associated person/user account
        User::delete(pool, application.user_id).await?;

        // Delete the application
        RegistrationApplication::delete(pool, application_id).await?;

        Ok(true)
    }

    /// Bulk approve multiple registration applications
    pub async fn bulk_approve_registration_applications(
        &self,
        ctx: &Context<'_>,
        application_ids: Vec<i32>,
    ) -> Result<i32> {
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

        let mut approved_count = 0;

        for application_id in application_ids {
            match RegistrationApplication::read(pool, application_id).await {
                Ok(application) => {
                    // Update the user to mark application as accepted
                    let user_form = UserForm {
                        is_application_accepted: Some(true),
                        accepted_application: Some(true),
                        ..UserForm::default()
                    };

                    if let Ok(user) = User::read(pool, application.user_id).await {
                        if User::update(pool, user.id, &user_form).await.is_ok() {
                            if RegistrationApplication::delete(pool, application_id).await.is_ok() {
                                approved_count += 1;
                            }
                        }
                    }
                }
                Err(_) => continue, // Skip invalid application IDs
            }
        }

        Ok(approved_count)
    }

    /// Bulk deny multiple registration applications  
    pub async fn bulk_deny_registration_applications(
        &self,
        ctx: &Context<'_>,
        application_ids: Vec<i32>,
    ) -> Result<i32> {
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

        let mut denied_count = 0;

        for application_id in application_ids {
            match RegistrationApplication::read(pool, application_id).await {
                Ok(application) => {
                    // Delete the associated person/user account
                    if User::delete(pool, application.user_id).await.is_ok() {
                        if RegistrationApplication::delete(pool, application_id).await.is_ok() {
                            denied_count += 1;
                        }
                    }
                }
                Err(_) => continue, // Skip invalid application IDs
            }
        }

        Ok(denied_count)
    }
}