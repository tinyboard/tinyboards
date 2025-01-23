use crate::helpers::validation::require_mod_or_admin;
use crate::structs::post::Post;
use crate::DbPool;
use crate::LoggedInUser;
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::models::person::local_user::AdminPerms;
use tinyboards_db::models::post::posts::Post as DbPost;
use tinyboards_db::traits::Crud;

#[derive(Default)]
pub struct PostModeration;

#[Object]
impl PostModeration {
    pub async fn set_post_removed(&self, ctx: &Context<'_>, id: i32, value: bool) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let post = DbPost::read(pool, id).await?;

        require_mod_or_admin(
            v,
            pool,
            post.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        DbPost::update_removed(pool, post.id, value).await?;
        // mark reports as resolved
        DbPost::resolve_reports(pool, post.id, v.person.id).await?;
        let res = DbPost::get_with_counts(pool, post.id, false).await?;

        Ok(Post::from(res))
    }

    pub async fn set_post_locked(&self, ctx: &Context<'_>, id: i32, value: bool) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;

        let post = DbPost::read(pool, id).await?;

        require_mod_or_admin(
            v,
            pool,
            post.board_id,
            ModPerms::Content,
            Some(AdminPerms::Content),
        )
        .await?;

        DbPost::update_locked(pool, post.id, value).await?;
        let res = DbPost::get_with_counts(pool, post.id, false).await?;

        Ok(Post::from(res))
    }
}
