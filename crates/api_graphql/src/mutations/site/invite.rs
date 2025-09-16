use async_graphql::*;
use tinyboards_db::{
    models::{
        user::user::AdminPerms,
        site::{
            site::Site as DbSite,
            site_invite::{SiteInvite as DbSiteInvite, SiteInviteForm},
        },
    },
    traits::Crud,
    utils::DbPool,
    RegistrationMode,
};
use tinyboards_utils::TinyBoardsError;
use uuid::Uuid;

use crate::LoggedInUser;

#[derive(Default)]
pub struct SiteInvite;

#[derive(SimpleObject)]
pub struct CreateInviteResponse {
    pub invite_code: String,
}

#[derive(SimpleObject)]
pub struct DeleteInviteResponse {
    pub success: bool,
}

#[Object]
impl SiteInvite {
    /// Create a site invite code
    pub async fn create_invite(&self, ctx: &Context<'_>) -> Result<CreateInviteResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let site = DbSite::read(pool).await?;
        let registration_mode = site.get_registration_mode();

        // Check if invites are enabled
        match registration_mode {
            RegistrationMode::InviteOnlyAdmin => {
                // Only admins can create invites
                if !user.has_permission(AdminPerms::Config) {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Only admins can create invites in this mode",
                    )
                    .into());
                }
            }
            RegistrationMode::InviteOnlyUser => {
                // Any user can create invites - no additional permission check needed
            }
            _ => {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Invite creation is not enabled for the current registration mode",
                )
                .into());
            }
        }

        // Generate a unique invite code
        let invite_code = Uuid::new_v4().to_string();

        let form = SiteInviteForm {
            verification_code: invite_code.clone(),
        };

        DbSiteInvite::create(pool, &form).await?;

        Ok(CreateInviteResponse { invite_code })
    }

    /// Delete a site invite code
    pub async fn delete_invite(&self, ctx: &Context<'_>, invite_id: i32) -> Result<DeleteInviteResponse> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let site = DbSite::read(pool).await?;
        let registration_mode = site.get_registration_mode();

        // Check if user can delete invites
        match registration_mode {
            RegistrationMode::InviteOnlyAdmin => {
                // Only admins can delete invites
                if !user.has_permission(AdminPerms::Config) {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Only admins can delete invites in this mode",
                    )
                    .into());
                }
            }
            RegistrationMode::InviteOnlyUser => {
                // Any user can delete invites - no additional permission check needed
            }
            _ => {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Invite deletion is not enabled for the current registration mode",
                )
                .into());
            }
        }

        // Verify the invite exists
        DbSiteInvite::read(pool, invite_id).await?;

        // Delete the invite
        let deleted_count = DbSiteInvite::delete(pool, invite_id).await?;

        Ok(DeleteInviteResponse {
            success: deleted_count > 0,
        })
    }
}