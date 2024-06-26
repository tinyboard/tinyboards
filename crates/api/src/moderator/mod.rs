use tinyboards_db::models::{comment::comments::Comment, post::posts::Post};
use tinyboards_db::traits::{Crud, Moderateable};
use tinyboards_db::utils::DbPool;
use tinyboards_utils::TinyBoardsError;

pub mod mod_ban_person;
//pub mod mod_remove_board;
pub mod mod_queue_comments;
pub mod mod_queue_posts;
pub mod mod_remove_object;

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
    pool: &DbPool,
    thing_fullname: &str,
) -> Result<Box<dyn Moderateable + Send>, TinyBoardsError> {
    use IdType::*;

    // we make sure we can safely split the fullname at the index of 3 by aborting if it's too short
    if thing_fullname.len() <= 3 {
        return Err(TinyBoardsError::from_message(400, "invalid fullname"));
    }

    // we split the t{n}_ thing and the id thing...
    let (id_type, id) = thing_fullname.split_at(3);
    // ...and try to convert the id from string to int
    let id = id
        .parse::<i32>()
        .map_err(|e| TinyBoardsError::from_error_message(e, 400, "invalid fullname"))?;

    // check if we're getting a comment or a post, disallow other things
    let id = match id_type {
        "t1_" => Ok(CommentId(id)),
        "t3_" => Ok(PostId(id)),
        _ => Err(TinyBoardsError::from_message(
            400,
            "This endpoint only accepts post and comment objects! Check if `target_fullname` begins with `t1_` or `t3_`.",
        )),
    }?;

    let target_object: Box<dyn Moderateable + Send> = match id {
        CommentId(id) => {
            let comment = Comment::read(pool, id).await?;

            Box::new(comment)
        }
        PostId(id) => {
            let post = Post::read(pool, id).await?;

            Box::new(post)
        }
    };

    Ok(target_object)
}
