use tinyboards_api_common::utils::blocking;
use tinyboards_db::database::PgPool;
use tinyboards_db::models::{comment::comments::Comment, post::posts::Post};
use tinyboards_db::traits::{Crud, Moderateable};
use tinyboards_utils::TinyBoardsError;

pub mod mod_add_admin;
pub mod mod_add_board_mod;
pub mod mod_ban_from_board;
pub mod mod_ban_user;
pub mod mod_lock_object;
pub mod mod_lock_post;
pub mod mod_remove_board;
pub mod mod_remove_comment;
pub mod mod_remove_object;
pub mod mod_remove_post;
pub mod mod_sticky_post;

// ;)
#[allow(dead_code)]
enum IdType {
    PostId(i32),
    CommentId(i32),
}

/**
Returns a Post or Comment from a fullname (both of them implement the `Moderateable` trait). fullname must start with either `t1_` (comments' prefix) or `t3_` (posts' prefix).
*/
pub(crate) async fn get_moderateable_object(
    pool: &PgPool,
    thing_fullname: &str,
) -> Result<Box<dyn Moderateable + Send>, TinyBoardsError> {
    use IdType::*;

    // we make sure we can safely split the fullname at the index of 3 by aborting if it's too short
    if thing_fullname.len() <= 3 {
        return Err(TinyBoardsError::from_message("invalid fullname"));
    }

    // we split the t{n}_ thing and the id thing...
    let (id_type, id) = thing_fullname.split_at(3);
    // ...and try to convert the id from string to int
    let id = id
        .parse::<i32>()
        .map_err(|e| TinyBoardsError::from_error_message(e, "invalid fullname"))?;

    // check if we're getting a comment or a post, disallow other things
    let id = match id_type {
        "t1_" => Ok(CommentId(id)),
        "t3_" => Ok(PostId(id)),
        _ => Err(TinyBoardsError::from_message(
            "This endpoint only accepts post and comment objects! Check if `target_fullname` begins with `t1_` or `t3_`.",
        )),
    }?;

    let target_object: Box<dyn Moderateable + Send> = match id {
        CommentId(id) => {
            let comment = blocking(pool, move |conn| Comment::read(conn, id)).await??;

            Box::new(comment)
        }
        PostId(id) => {
            let post = blocking(pool, move |conn| Post::read(conn, id)).await??;

            Box::new(post)
        }
    };

    Ok(target_object)
}
