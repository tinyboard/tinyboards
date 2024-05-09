use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    emoji::{DeleteEmoji, DeleteEmojiResponse, EmojiIdPath},
    utils::require_user,
};
use tinyboards_db::models::{
    emoji::{emoji::Emoji, emoji_keyword::EmojiKeyword},
    person::local_user::AdminPerms,
};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for DeleteEmoji {
    type Response = DeleteEmojiResponse;
    type Route = EmojiIdPath;

    #[tracing::instrument(skip(self, context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        auth: Option<&str>,
    ) -> Result<DeleteEmojiResponse, TinyBoardsError> {
        // only admins should be deleting emojis
        let _view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Config)
            .unwrap()?;

        EmojiKeyword::delete(context.pool(), path.emoji_id.clone()).await?;
        Emoji::delete(context.pool(), path.emoji_id.clone()).await?;

        Ok(DeleteEmojiResponse {
            id: path.emoji_id.clone(),
            success: true,
        })
    }
}
