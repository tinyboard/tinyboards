use async_graphql::*;
use tinyboards_db::{
    models::{
        person::local_user::AdminPerms,
        site::{
            local_site::LocalSite as DbLocalSite,
            site_invite::SiteInvite as DbSiteInvite,
        },
    },
    utils::DbPool,
    RegistrationMode,
};
use tinyboards_utils::TinyBoardsError;

use crate::LoggedInUser;

#[derive(Default)]
pub struct QueryInvites;

#[derive(SimpleObject)]
pub struct Invite {
    pub id: i32,
    pub verification_code: String,
    pub created: String,
}

#[Object]
impl QueryInvites {
    /// List site invites
    pub async fn list_invites(&self, ctx: &Context<'_>) -> Result<Vec<Invite>> {
        let pool = ctx.data::<DbPool>()?;
        let user = ctx.data_unchecked::<LoggedInUser>().require_user()?;

        let site = DbLocalSite::read(pool).await?;
        let registration_mode = site.get_registration_mode();

        // Check if user can view invites
        match registration_mode {
            RegistrationMode::InviteOnlyAdmin => {
                // Only admins can view invites
                if !user.has_permission(AdminPerms::Config) {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Only admins can view invites in this mode",
                    )
                    .into());
                }
            }
            RegistrationMode::InviteOnlyUser => {
                // Any user can view invites - no additional permission check needed
            }
            _ => {
                return Err(TinyBoardsError::from_message(
                    400,
                    "Invite viewing is not enabled for the current registration mode",
                )
                .into());
            }
        }

        let invites = DbSiteInvite::read_all(pool).await?;

        let result = invites
            .into_iter()
            .map(|invite| Invite {
                id: invite.id,
                verification_code: invite.verification_code,
                created: invite.created.to_string(),
            })
            .collect();

        Ok(result)
    }
}