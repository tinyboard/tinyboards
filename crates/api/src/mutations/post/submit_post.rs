use crate::helpers::files::upload::upload_file_opendal;
use crate::structs::post::Post;
use crate::utils::emoji::process_content_with_emojis;
use crate::{DbPool, LoggedInUser, Settings};
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::models::user::user::AdminPerms;
use tinyboards_db::models::{
    board::boards::Board as DbBoard,
    post::{
        post_votes::{PostVote, PostVoteForm},
        posts::{Post as DbPost, PostForm},
    },
};
use tinyboards_db::traits::Crud;
use tinyboards_db::traits::Voteable;
use tinyboards_utils::{
    content_filter::ContentFilter, parser::parse_markdown_opt, utils::custom_body_parsing,
    TinyBoardsError,
};
use url::Url;

#[derive(Default)]
pub struct SubmitPost;

#[Object]
impl SubmitPost {
    pub async fn create_post(
        &self,
        ctx: &Context<'_>,
        title: String,
        board: Option<String>,
        body: Option<String>,
        link: Option<String>,
        #[graphql(name = "isNSFW")] is_nsfw: Option<bool>,
        alt_text: Option<String>,
        file: Option<Upload>,
        post_type: Option<String>,
    ) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        // Load site configuration for content filtering and emoji settings
        let site_config = tinyboards_db::models::site::site::Site::read(pool).await?;

        let board = match board {
            Some(ref board) => DbBoard::get_by_name(pool, board).await?,
            None => DbBoard::read(pool, 1).await?,
        };

        if board.is_removed || board.is_deleted {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("/b/{} is banned.", &board.name),
            )
            .into());
        }

        if board.is_banned {
            let reason = board
                .public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        // mod or admin check
        let is_mod_or_admin = if v.has_permission(AdminPerms::Content) {
            // user is admin
            true
        } else {
            // user is not admin: check mod permissions instead
            let m = DbBoard::board_get_mod(pool, board.id, v.id).await;

            match m {
                Ok(m_opt) => match m_opt {
                    Some(m) => m.has_permission(ModPerms::Content),
                    None => false,
                },
                Err(e) => {
                    eprintln!("Error while checking mod permissions: {:?}", e);
                    false
                }
            }
        };

        // check if user is banned from board
        if !is_mod_or_admin && DbBoard::board_has_ban(pool, board.id, v.id).await? {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("You are banned from /b/{}.", &board.name),
            )
            .into());
        }

        let body_html = match body {
            Some(ref body) => {
                if site_config.emoji_enabled {
                    // Process content with emoji parsing (use site config limit)
                    let emoji_limit = site_config.max_emojis_per_post.map(|limit| limit as usize);
                    let processed_html = process_content_with_emojis(
                        body,
                        pool,
                        Some(board.id),
                        settings,
                        emoji_limit,
                    )
                    .await?;

                    Some(processed_html)
                } else {
                    // Emojis disabled, use regular markdown processing
                    let body_html = parse_markdown_opt(body);
                    Some(custom_body_parsing(
                        &body_html.unwrap_or_default(),
                        settings,
                    ))
                }
            }
            None => Some(String::new()),
        };

        let type_ = if file.is_some() {
            "image".to_owned()
        } else if link.is_some() {
            "link".to_owned()
        } else {
            "text".to_owned()
        };

        // Validate content against site policies
        ContentFilter::validate_post_content(
            &site_config.allowed_post_types,
            &site_config.word_filter_enabled,
            &site_config.word_filter_applies_to_posts,
            &site_config.filtered_words,
            &site_config.link_filter_enabled,
            &site_config.banned_domains,
            &type_,
            &title,
            &body,
            &link,
        )?;

        // Check NSFW tagging requirement
        if site_config.enable_nsfw_tagging.unwrap_or(true)
            && site_config.enable_nsfw
            && is_nsfw.unwrap_or(false)
        {
            // NSFW content is allowed and user tagged it as NSFW - this is fine
        } else if !site_config.enable_nsfw && is_nsfw.unwrap_or(false) {
            return Err(TinyBoardsError::from_message(
                403,
                "NSFW content is not allowed on this site",
            )
            .into());
        }

        //let data_url = data.url.as_ref().map(|url| url.inner());
        let is_nsfw = is_nsfw.unwrap_or(false);
        let url = match &link {
            Some(link) => Some(Url::parse(link)?),
            None => None,
        };

        // Determine post type (feed or thread, default to feed)
        let determined_post_type = post_type.unwrap_or_else(|| "feed".to_string());

        // Validate post_type
        if determined_post_type != "feed" && determined_post_type != "thread" {
            return Err(TinyBoardsError::from_message(
                400,
                "post_type must be either 'feed' or 'thread'"
            ).into());
        }

        let post_form = PostForm {
            title: Some(title.clone()),
            type_: Some(type_),
            url: url.map(|url| url.into()),
            // image: data.image,
            body: Some(body.unwrap_or_default()), // Ensure body is always Some(String)
            body_html: body_html,
            creator_id: Some(v.id),
            board_id: Some(board.id),
            is_nsfw: Some(is_nsfw),
            alt_text: alt_text,
            title_chunk: Some(DbPost::generate_chunk(title)),
            is_removed: Some(false),
            is_locked: Some(false),
            is_deleted: Some(false),
            featured_board: Some(false),
            featured_local: Some(false),
            post_type: Some(determined_post_type.clone()),
            creator_vote: Some(if determined_post_type == "thread" { 0 } else { 1 }), // No self-vote for threads
            ..Default::default()
        };

        let published_post = DbPost::submit(pool, post_form).await?;

        // handle file upload
        let file_url = match file {
            Some(file) => Some(upload_file_opendal(file, None, v.id, None, ctx).await?),
            None => None,
        };

        // Check URL image links if it's a link post with an image
        if let Some(ref url_str) = link {
            let url_string = url_str.to_string();
            if url_string.ends_with(".jpg")
                || url_string.ends_with(".jpeg")
                || url_string.ends_with(".png")
                || url_string.ends_with(".gif")
                || url_string.ends_with(".webp")
            {
                if !ContentFilter::is_image_host_approved(
                    &site_config.approved_image_hosts,
                    &site_config.image_embed_hosts_only,
                    &url_string,
                )? {
                    return Err(TinyBoardsError::from_message(
                        403,
                        "Image links from this host are not allowed",
                    )
                    .into());
                }
            }
        }

        // If image was uploaded, update post with image URL
        if file_url.is_some() {
            let post_update_form = PostForm {
                url: file_url.clone().map(|url| url.into()),
                image: file_url.map(|url| url.into()),
                ..PostForm::default()
            };

            DbPost::update(pool, published_post.id.clone(), &post_update_form)
                .await
                .map_err(|e| {
                    TinyBoardsError::from_error_message(
                        e,
                        500,
                        "Failed to update post with file URL",
                    )
                })?;
        }

        // auto upvote own post (only for feed posts, not threads)
        if determined_post_type == "feed" {
            let post_vote = PostVoteForm {
                post_id: published_post.id,
                user_id: v.id,
                score: 1,
            };

            PostVote::vote(pool, &post_vote).await?;
        }

        let post_with_counts = DbPost::get_with_counts(pool, published_post.id, false)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch post"))?;

        Ok(Post::from(post_with_counts))
    }
}
