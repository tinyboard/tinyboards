use crate::helpers::files::upload::upload_file_opendal;
use crate::helpers::files::cleanup::link_content_uploads;
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
    content_filter::ContentFilter, parser::{parse_markdown_opt, sanitize_html}, utils::custom_body_parsing,
    slug::{generate_slug, ensure_unique_slug},
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
        let pool = ctx.data::<DbPool>()?;
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_approved(pool)
            .await?;
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

        // Determine if this is a thread post (HTML from rich editor) or feed post (markdown)
        let is_thread_post = post_type.as_deref() == Some("thread");

        // Extract mentions from title and body BEFORE they get moved
        // We'll send notifications later after the post is created
        let mut all_text_for_mentions = title.clone();
        if let Some(ref body_text) = body {
            all_text_for_mentions.push(' ');
            all_text_for_mentions.push_str(body_text);
        }

        let body_html = match body {
            Some(ref body) => {
                if is_thread_post {
                    // Thread posts use rich text editor - body is already HTML
                    // Sanitize, parse custom emojis, then custom body parsing
                    let sanitized = sanitize_html(body);
                    let with_emojis = if site_config.emoji_enabled {
                        let parser = crate::utils::emoji::EmojiParser::new(pool, Some(board.id)).await?;
                        let emoji_limit = site_config.max_emojis_per_post.map(|limit| limit as usize);
                        parser.validate_emoji_usage(&sanitized, emoji_limit)?;
                        let html_with_emojis = parser.parse_emojis_to_html(&sanitized);
                        // Increment usage in background
                        let _task = parser.increment_emoji_usage(&html_with_emojis, pool);
                        html_with_emojis
                    } else {
                        sanitized
                    };
                    let processed = custom_body_parsing(&with_emojis, settings);

                    // Convert @mentions to links
                    use crate::helpers::notifications::convert_mentions_to_links;
                    let with_mention_links = convert_mentions_to_links(&processed);

                    Some(with_mention_links)
                } else {
                    // Feed posts use markdown - convert then sanitize
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

                        // Convert @mentions to links before sanitizing
                        use crate::helpers::notifications::convert_mentions_to_links;
                        let with_mention_links = convert_mentions_to_links(&processed_html);

                        Some(sanitize_html(&with_mention_links))
                    } else {
                        // Emojis disabled, use regular markdown processing
                        let body_html = parse_markdown_opt(body);
                        let processed = custom_body_parsing(
                            &body_html.unwrap_or_default(),
                            settings,
                        );

                        // Convert @mentions to links
                        use crate::helpers::notifications::convert_mentions_to_links;
                        let with_mention_links = convert_mentions_to_links(&processed);

                        Some(sanitize_html(&with_mention_links))
                    }
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

        // Generate slug from title
        let base_slug = generate_slug(&title, Some(60));

        // Ensure slug is unique within board context
        use tinyboards_db::schema::posts;
        use diesel::dsl::*;
        use diesel_async::RunQueryDsl;

        let unique_slug = ensure_unique_slug(&base_slug, |slug| {
            // Check if slug already exists in this board
            let pool = pool.clone();
            let slug = slug.to_string();
            let board_id = board.id;

            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let conn = &mut tinyboards_db::utils::get_conn(&pool).await.ok()?;
                    let count: i64 = posts::table
                        .filter(posts::board_id.eq(board_id))
                        .filter(posts::slug.eq(&slug))
                        .count()
                        .get_result(conn)
                        .await
                        .ok()?;
                    Some(count > 0)
                })
            }).unwrap_or(false)
        });

        let post_form = PostForm {
            title: Some(title.clone()),
            url: url.map(|url| url.into()),
            // image: data.image,
            body: Some(body.unwrap_or_default()), // Ensure body is always Some(String)
            body_html: body_html.clone(),
            creator_id: Some(v.id),
            board_id: Some(board.id),
            is_nsfw: Some(is_nsfw),
            alt_text: alt_text,
            title_chunk: Some(DbPost::generate_chunk(title)),
            is_removed: Some(false),
            is_locked: Some(false),
            featured_board: Some(false),
            featured_local: Some(false),
            type_: Some(determined_post_type.clone()),
            creator_vote: Some(if determined_post_type == "thread" { 0 } else { 1 }), // No self-vote for threads
            slug: Some(unique_slug),
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

        // Link any uploaded images found in the HTML content
        if let Some(ref html) = body_html {
            if !html.is_empty() {
                link_content_uploads(pool, published_post.id, true, html).await?;
            }
        }

        // Send notifications for mentions in post
        use crate::helpers::notifications::{
            extract_mentions, get_user_ids_for_mentions, create_post_mention_notification
        };

        let mentions = extract_mentions(&all_text_for_mentions);

        if !mentions.is_empty() {
            // Get user IDs for mentioned usernames
            if let Ok(mentioned_user_ids) = get_user_ids_for_mentions(pool, mentions).await {
                for mentioned_user_id in mentioned_user_ids {
                    // Don't notify yourself
                    if mentioned_user_id != v.id {
                        let _ = create_post_mention_notification(
                            pool,
                            mentioned_user_id,
                            published_post.id,
                        ).await; // Ignore errors for notifications
                    }
                }
            }
        }

        let post_with_counts = DbPost::get_with_counts(pool, published_post.id, false)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to fetch post"))?;

        Ok(Post::from(post_with_counts))
    }
}
