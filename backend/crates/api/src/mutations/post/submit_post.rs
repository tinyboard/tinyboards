use crate::helpers::files::upload::upload_file_opendal;
use crate::helpers::files::cleanup::link_content_uploads;
use crate::helpers::permissions;
use crate::structs::post::Post;
use crate::{DbPool, LoggedInUser, Settings};
use async_graphql::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tinyboards_db::{
    enums::{DbApprovalStatus, DbPostType},
    models::{
        aggregates::PostAggregates,
        board::board_mods::{BoardModerator, ModPerms},
        board::boards::Board as DbBoard,
        post::post_votes::PostVoteInsertForm,
        post::posts::{Post as DbPost, PostInsertForm},
        site::site::Site,
        user::user::AdminPerms,
    },
    schema::{
        board_moderators, board_user_bans, boards, post_aggregates, posts, site,
    },
    utils::get_conn,
};
use tinyboards_utils::{
    content_filter::ContentFilter,
    slug::generate_slug,
    TinyBoardsError,
};
use url::Url;
use uuid::Uuid;
use rand::{thread_rng, Rng};

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
        let conn = &mut get_conn(pool).await?;

        // Load site configuration
        let site_config: Site = site::table
            .first(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Resolve board
        let db_board: DbBoard = match board {
            Some(ref board_name) => boards::table
                .filter(boards::name.eq(board_name))
                .filter(boards::deleted_at.is_null())
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound(format!("Board '{}' not found", board_name)))?,
            None => boards::table
                .filter(boards::deleted_at.is_null())
                .first(conn)
                .await
                .map_err(|_| TinyBoardsError::NotFound("No default board found".into()))?,
        };

        if db_board.is_removed || db_board.deleted_at.is_some() {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("/b/{} is deleted.", &db_board.name),
            )
            .into());
        }

        if db_board.is_banned {
            let reason = db_board
                .public_ban_reason
                .as_deref()
                .unwrap_or("This board has been banned");
            return Err(TinyBoardsError::from_message(403, reason).into());
        }

        // mod or admin check
        let is_mod_or_admin = if v.has_permission(AdminPerms::Content) {
            true
        } else {
            board_moderators::table
                .filter(board_moderators::board_id.eq(db_board.id))
                .filter(board_moderators::user_id.eq(v.id))
                .first::<BoardModerator>(conn)
                .await
                .ok()
                .map(|m| m.has_permission(ModPerms::Content))
                .unwrap_or(false)
        };

        // check if user is banned from board
        if !is_mod_or_admin {
            let banned: bool = board_user_bans::table
                .filter(board_user_bans::board_id.eq(db_board.id))
                .filter(board_user_bans::user_id.eq(v.id))
                .first::<tinyboards_db::models::social::BoardUserBan>(conn)
                .await
                .is_ok();
            if banned {
                return Err(TinyBoardsError::from_message(
                    403,
                    &format!("You are banned from /b/{}.", &db_board.name),
                )
                .into());
            }
        }

        // Extract mentions from title and body
        let mut all_text_for_mentions = title.clone();
        if let Some(ref body_text) = body {
            all_text_for_mentions.push(' ');
            all_text_for_mentions.push_str(body_text);
        }

        let emoji_limit = if site_config.emoji_enabled {
            site_config.max_emojis_per_post.map(|limit| limit as usize)
        } else {
            Some(0)
        };

        let body_html = match body {
            Some(ref body_text) => {
                crate::utils::emoji::process_content_with_emojis(
                    body_text,
                    pool,
                    Some(db_board.id),
                    settings,
                    emoji_limit,
                )
                .await?
            }
            None => String::new(),
        };

        // Determine post type enum
        let db_post_type = if file.is_some() {
            DbPostType::Image
        } else if link.is_some() {
            DbPostType::Link
        } else {
            DbPostType::Text
        };

        let type_str = match db_post_type {
            DbPostType::Image => "image",
            DbPostType::Link => "link",
            DbPostType::Text => "text",
            DbPostType::Video => "video",
        };

        // Validate content against site policies
        ContentFilter::validate_post_content(
            &site_config.allowed_post_types,
            &Some(site_config.word_filter_enabled),
            &Some(site_config.word_filter_applies_to_posts),
            &site_config.filtered_words,
            &Some(site_config.link_filter_enabled),
            &site_config.banned_domains,
            type_str,
            &title,
            &body,
            &link,
        )?;

        // Check NSFW
        let is_nsfw = is_nsfw.unwrap_or(false);
        if !site_config.enable_nsfw && is_nsfw {
            return Err(TinyBoardsError::from_message(
                403,
                "NSFW content is not allowed on this site",
            )
            .into());
        }

        let url = match &link {
            Some(link_str) => Some(Url::parse(link_str)?.to_string()),
            None => None,
        };

        // Validate post type against board mode
        use tinyboards_db::enums::DbBoardMode;
        let determined_post_type_str = post_type.unwrap_or_else(|| {
            match db_board.mode {
                DbBoardMode::Feed => "feed".to_string(),
                DbBoardMode::Forum => "thread".to_string(),
            }
        });
        match db_board.mode {
            DbBoardMode::Feed => {
                if determined_post_type_str == "thread" {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!(
                            "/b/{} is a Feed board and does not accept thread posts. Submit a feed post instead.",
                            &db_board.name
                        ),
                    )
                    .into());
                }
            }
            DbBoardMode::Forum => {
                if determined_post_type_str != "thread" {
                    return Err(TinyBoardsError::from_message(
                        400,
                        &format!(
                            "/b/{} is a Forum board and only accepts thread posts. Submit a thread instead.",
                            &db_board.name
                        ),
                    )
                    .into());
                }
            }
        }

        // Generate unique slug
        let base_slug = generate_slug(&title, Some(60));
        let mut unique_slug = base_slug.clone();
        let mut counter = 2;

        loop {
            let count: i64 = posts::table
                .filter(posts::board_id.eq(db_board.id))
                .filter(posts::slug.eq(&unique_slug))
                .count()
                .get_result(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

            if count == 0 {
                break;
            }

            unique_slug = format!("{}-{}", base_slug, counter);
            counter += 1;

            if counter > 1000 {
                let random_suffix: u32 = thread_rng().gen();
                unique_slug = format!("{}-{}", base_slug, random_suffix);
                break;
            }
        }

        // Check URL image links if it's a link post
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
                    &Some(site_config.image_embed_hosts_only),
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

        let post_id = Uuid::new_v4();

        let post_form = PostInsertForm {
            id: post_id,
            title: title.clone(),
            post_type: db_post_type,
            url,
            thumbnail_url: None,
            body: body.clone().unwrap_or_default(),
            body_html: body_html.clone(),
            image: None, // set below if file uploaded
            alt_text,
            slug: unique_slug,
            creator_id: v.id,
            board_id: db_board.id,
            language_id: None,
            is_nsfw,
            approval_status: DbApprovalStatus::Approved,
            embed_title: None,
            embed_description: None,
            embed_video_url: None,
            source_url: None,
            is_thread: determined_post_type_str == "thread",
        };

        diesel::insert_into(posts::table)
            .values(&post_form)
            .execute(conn)
            .await
            .map_err(|e| TinyBoardsError::Database(e.to_string()))?;

        // Handle file upload
        if let Some(file) = file {
            let file_url = upload_file_opendal(file, None, v.id, None, ctx).await?;
            diesel::update(posts::table.find(post_id))
                .set(posts::image.eq(Some(file_url.to_string())))
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }

        // Auto upvote own post (feed posts only)
        if determined_post_type_str != "thread" {
            let vote_form = PostVoteInsertForm {
                id: Uuid::new_v4(),
                user_id: v.id,
                post_id: post_id,
                score: 1,
            };
            diesel::insert_into(tinyboards_db::schema::post_votes::table)
                .values(&vote_form)
                .execute(conn)
                .await
                .map_err(|e| TinyBoardsError::Database(e.to_string()))?;
        }

        // Link any uploaded images found in the HTML content
        if !body_html.is_empty() {
            link_content_uploads(pool, post_id, true, &body_html).await?;
        }

        // Send notifications for mentions
        use crate::helpers::notifications::{
            extract_mentions, get_user_ids_for_mentions, create_post_mention_notification,
        };

        let mentions = extract_mentions(&all_text_for_mentions);
        if !mentions.is_empty() {
            if let Ok(mentioned_user_ids) = get_user_ids_for_mentions(pool, mentions).await {
                for mentioned_user_id in mentioned_user_ids {
                    if mentioned_user_id != v.id {
                        let _ = create_post_mention_notification(
                            pool,
                            mentioned_user_id,
                            post_id,
                            v.id,
                        )
                        .await;
                    }
                }
            }
        }

        // Load the created post with aggregates
        let db_post: DbPost = posts::table
            .find(post_id)
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post not found after creation".into()))?;

        let agg: PostAggregates = post_aggregates::table
            .filter(post_aggregates::post_id.eq(post_id))
            .first(conn)
            .await
            .map_err(|_| TinyBoardsError::NotFound("Post aggregates not found".into()))?;

        Ok(Post::from((db_post, agg)))
    }
}
