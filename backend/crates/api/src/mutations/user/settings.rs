use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    models::user::user::{User as DbUser, UserUpdateForm},
    schema::users,
    utils::{get_conn, DbPool},
};
use tinyboards_utils::TinyBoardsError;

use crate::{helpers::permissions, structs::user::UserSettings};

#[derive(Default)]
pub struct UpdateSettings;

#[derive(InputObject)]
pub struct UpdateSettingsInput {
    pub theme: Option<String>,
    pub show_nsfw: Option<bool>,
    pub show_bots: Option<bool>,
    pub interface_language: Option<String>,
    pub is_email_notifications_enabled: Option<bool>,
}

#[Object]
impl UpdateSettings {
    /// Update user preferences/settings.
    pub async fn update_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateSettingsInput,
    ) -> Result<UserSettings> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        let form = UserUpdateForm {
            theme: input.theme,
            show_nsfw: input.show_nsfw,
            show_bots: input.show_bots,
            interface_language: input.interface_language,
            is_email_notifications_enabled: input.is_email_notifications_enabled,
            ..Default::default()
        };

        let updated: DbUser = diesel::update(users::table.find(me.id))
            .set(&form)
            .get_result(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(UserSettings::from(updated))
    }

    /// Soft delete the current user's account.
    pub async fn delete_account(&self, ctx: &Context<'_>) -> Result<bool> {
        let me = permissions::require_auth_not_banned(ctx)?;
        let pool = ctx.data::<DbPool>()?;
        let conn = &mut get_conn(pool).await?;

        diesel::update(users::table.find(me.id))
            .set(users::deleted_at.eq(diesel::dsl::now))
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        Ok(true)
    }
}
