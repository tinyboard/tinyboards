use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::user::user::{User as DbUser, UserUpdateForm},
    schema::users,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use crate::{helpers::permissions, structs::user::User};

#[derive(Default)]
pub struct ProfileManagement;

#[derive(InputObject)]
pub struct UpdateProfileInput {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub profile_background: Option<String>,
    pub avatar_frame: Option<String>,
    pub profile_music: Option<String>,
    pub profile_music_youtube: Option<String>,
    pub signature: Option<String>,
}

#[Object]
impl ProfileManagement {
    /// Update user profile. Uses if-let instead of boolean flags + unwrap (BUG-033 fix).
    pub async fn update_profile(
        &self,
        ctx: &Context<'_>,
        input: UpdateProfileInput,
    ) -> Result<User> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        // Build the update form using if-let pattern (BUG-033 fix)
        let form = UserUpdateForm {
            display_name: if let Some(ref v) = input.display_name { Some(Some(v.clone())) } else { None },
            bio: if let Some(ref v) = input.bio { Some(Some(v.clone())) } else { None },
            avatar: if let Some(ref v) = input.avatar { Some(Some(v.clone())) } else { None },
            banner: if let Some(ref v) = input.banner { Some(Some(v.clone())) } else { None },
            profile_background: if let Some(ref v) = input.profile_background { Some(Some(v.clone())) } else { None },
            avatar_frame: if let Some(ref v) = input.avatar_frame { Some(Some(v.clone())) } else { None },
            profile_music: if let Some(ref v) = input.profile_music { Some(Some(v.clone())) } else { None },
            profile_music_youtube: if let Some(ref v) = input.profile_music_youtube { Some(Some(v.clone())) } else { None },
            signature: if let Some(ref v) = input.signature { Some(Some(v.clone())) } else { None },
            ..Default::default()
        };

        let updated: DbUser = diesel::update(users::table.find(me.id))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(User::from_db(updated, None))
    }
}
