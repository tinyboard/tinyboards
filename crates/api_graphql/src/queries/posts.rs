use crate::helpers::validation::check_private_instance;
use crate::Censorable;
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::{
    models::{
        board::board_mods::BoardModerator as DbBoardMod,
        board::boards::Board as DbBoard,
        person::{local_user::AdminPerms, person::Person as DbPerson},
        post::posts::Post as DbPost,
    },
    utils::DbPool,
};
use tinyboards_utils::TinyBoardsError;

use crate::{structs::post::Post, ListingType, LoggedInUser, SortType};

#[derive(Default)]
pub struct QueryPosts;

#[Object]
impl QueryPosts {
    pub async fn post(&self, ctx: &Context<'_>, id: i32) -> Result<Post> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let require_board_not_banned = match v_opt {
            Some(v) => !v.local_user.has_permission(AdminPerms::Boards),
            None => true,
        };

        let res = DbPost::get_with_counts(pool, id, require_board_not_banned)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 404, "Post not found"))?;

        let mut post = Post::from(res);
        let is_admin = match v_opt {
            Some(v) => v.local_user.has_permission(AdminPerms::Content),
            None => false,
        };
        let is_mod = match v_opt {
            Some(v) => {
                let mod_rel =
                    DbBoardMod::get_by_person_id_for_board(pool, v.person.id, post.board_id, true)
                        .await;
                match mod_rel {
                    Ok(m) => m.has_permission(ModPerms::Content),
                    Err(_) => false,
                }
            }
            None => false,
        };

        let my_person_id = v_opt.as_ref().map(|v| v.person.id).unwrap_or(-1);
        if !is_admin {
            post.censor(my_person_id, is_admin, is_mod);
        }

        Ok(post)
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
        person_name: Option<String>,
        board_name: Option<String>,
        #[graphql(desc = "Whether to only show saved posts.")] saved_only: Option<bool>,
        #[graphql(desc = "Page.")] page: Option<i64>,
    ) -> Result<Vec<Post>> {
        let pool = ctx.data::<DbPool>()?;
        let v_opt = ctx.data::<LoggedInUser>()?.inner();

        check_private_instance(v_opt, pool).await?;

        let sort = sort.unwrap_or(SortType::NewComments);
        let listing_type = listing_type.unwrap_or(ListingType::Local);
        let limit = std::cmp::min(limit.unwrap_or(25), 25);
        let person_id_join = match v_opt {
            Some(v) => v.person.id,
            None => -1,
        };

        let person_id = match person_name {
            Some(name) => DbPerson::get_by_name(pool, name)
                .await
                .map(|u| Some(u.id))
                .unwrap_or(Some(0)),
            None => person_id,
        };

        let board_id = match board_name {
            Some(name) => DbBoard::get_by_name(pool, name.as_str())
                .await
                .map(|b| Some(b.id))
                .unwrap_or(Some(0)),
            None => board_id,
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
