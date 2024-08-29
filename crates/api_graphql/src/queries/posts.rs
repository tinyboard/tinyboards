use async_graphql::*;
use tinyboards_db::{
    models::{person::local_user::AdminPerms, post::posts::Post as DbPost},
    utils::DbPool,
};
use tinyboards_db_views::post_view::PostQuery;
use tinyboards_utils::TinyBoardsError;

use crate::{structs::post::Post, ListingType, LoggedInUser, SortType};

#[derive(Default)]
pub struct QueryPosts;

#[Object]
impl QueryPosts {
    pub async fn post(&self, ctx: &Context<'_>, id: i32) -> Result<Post> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let require_board_not_banned = match v_opt {
            Some(v) => !v.local_user.has_permission(AdminPerms::Boards),
            None => true,
        };

        let res = DbPost::get_with_counts(pool, id, require_board_not_banned)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Post not found"))?;

        Ok(Post::from(res))
    }

    pub async fn list_posts<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(desc = "Limit of how many posts to load. Max value and default is 25.")]
        limit: Option<i64>,
        #[graphql(desc = "Sorting type.")] sort: Option<SortType>,
        #[graphql(desc = "Listing type, eg. \"Local\" or \"Subscribed\".")] listing_type: Option<
            ListingType,
        >,
        #[graphql(desc = "If specified, only posts from the given user will be loaded.")]
        person_id: Option<i32>,
        #[graphql(desc = "If specified, only posts in the given board will be loaded.")]
        board_id: Option<i32>,
        #[graphql(desc = "Whether to only show saved posts.")] saved_only: Option<bool>,
        #[graphql(desc = "Page.")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        let sort = sort.unwrap_or(SortType::NewComments);
        let listing_type = listing_type.unwrap_or(ListingType::Local);
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };

        let posts = DbPost::load_with_counts(
            pool,
            person_id_join,
            Some(limit),
            page,
            false,
            false,
            false,
            saved_only.unwrap_or(false),
            board_id,
            person_id,
            sort.into(),
            listing_type.into(),
        )
        .await?;

        Ok(posts.into_iter().map(Post::from).collect::<Vec<Post>>())
    }
}
