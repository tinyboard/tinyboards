use crate::helpers::apub::EndpointType;
use crate::helpers::{apub::generate_local_apub_endpoint, files::upload::upload_file};
use crate::structs::post::Post;
use crate::{DbPool, LoggedInUser, Settings};
use async_graphql::*;
use tinyboards_db::models::board::board_mods::ModPerms;
use tinyboards_db::models::person::local_user::AdminPerms;
use tinyboards_db::models::{
    board::boards::Board as DbBoard,
    post::{
        post_votes::{PostVote, PostVoteForm},
        posts::{Post as DbPost, PostForm},
    },
};
use tinyboards_db::traits::Crud;
use tinyboards_db::traits::Voteable;
use tinyboards_utils::{parser::parse_markdown_opt, utils::custom_body_parsing, TinyBoardsError};
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
        file: Option<Upload>,
    ) -> Result<Post> {
        let v = ctx
            .data_unchecked::<LoggedInUser>()
            .require_user_not_banned()?;
        let pool = ctx.data::<DbPool>()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        let board = match board {
            Some(ref board) => DbBoard::get_by_name(pool, board).await?,
            None => DbBoard::read(pool, 1).await?,
        };

        if board.is_removed || board.is_deleted {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("+{} is banned.", &board.name),
            )
            .into());
        }

        // mod or admin check
        let is_mod_or_admin = if v.local_user.has_permission(AdminPerms::Content) {
            // user is admin
            true
        } else {
            // user is not admin: check mod permissions instead
            let m = DbBoard::board_get_mod(pool, board.id, v.person.id).await;

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
        if !is_mod_or_admin && DbBoard::board_has_ban(pool, board.id, v.person.id).await? {
            return Err(TinyBoardsError::from_message(
                410,
                &format!("You are banned from +{}.", &board.name),
            )
            .into());
        }

        let body_html = match body {
            Some(ref body) => {
                let body_html = parse_markdown_opt(body);
                let body_html = Some(custom_body_parsing(
                    &body_html.unwrap_or_default(),
                    settings,
                ));

                body_html
            }
            None => Some(String::new()),
        };

        let type_ = if link.is_some() { "link" } else { "text" }.to_owned();

        //let data_url = data.url.as_ref().map(|url| url.inner());
        let is_nsfw = is_nsfw.unwrap_or(false);
        let url = match link {
            Some(link) => Some(Url::parse(&link)?),
            None => None,
        };

        let post_form = PostForm {
            title: Some(title.clone()),
            type_: Some(type_),
            url: url.map(|url| url.into()),
            // image: data.image,
            body: body, // once told me, the world was gonna roll me
            body_html: body_html,
            creator_id: Some(v.person.id),
            board_id: Some(board.id),
            is_nsfw: Some(is_nsfw),
            title_chunk: Some(DbPost::generate_chunk(title)),
            ..PostForm::default()
        };

        let published_post = DbPost::submit(pool, post_form).await?;

        // apub id add
        let protocol_and_hostname = settings.get_protocol_and_hostname();
        let apub_id = generate_local_apub_endpoint(
            EndpointType::Post,
            &published_post.id.clone().to_string(),
            &protocol_and_hostname,
        )?;

        // handle file upload
        let file_url = match file {
            Some(file) => Some(upload_file(file, None, v.person.id, ctx).await?),
            None => None,
        };

        // do not override url unless image is uplaoded
        let update_form = if file_url.is_some() {
            PostForm {
                ap_id: Some(apub_id),
                url: file_url.clone().map(|url| url.into()),
                image: file_url.map(|url| url.into()),
                ..PostForm::default()
            }
        } else {
            PostForm {
                ap_id: Some(apub_id),
                ..PostForm::default()
            }
        };

        let updated_post = DbPost::update(pool, published_post.id.clone(), &update_form).await?;

        // auto upvote own post
        let post_vote = PostVoteForm {
            post_id: updated_post.id,
            person_id: v.person.id,
            score: 1,
        };

        PostVote::vote(pool, &post_vote).await?;

        let post_with_counts = DbPost::get_with_counts(pool, published_post.id, false).await?;

        Ok(Post::from(post_with_counts))
    }
}
