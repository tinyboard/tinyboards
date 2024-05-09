use crate::PerformCrud;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    emoji::{EditEmoji, EmojiResponse},
    utils::require_user,
};
use tinyboards_db::{
    models::{
        emoji::{
            emoji::{Emoji, EmojiForm},
            emoji_keyword::{EmojiKeyword, EmojiKeywordForm},
        },
        person::local_user::AdminPerms,
        site::local_site::LocalSite,
    },
    utils::naive_now,
};
use tinyboards_db_views::structs::EmojiView;
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> PerformCrud<'des> for EditEmoji {
    type Response = EmojiResponse;
    type Route = ();

    #[tracing::instrument(skip(self, context, auth))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        _: Self::Route,
        auth: Option<&str>,
    ) -> Result<EmojiResponse, TinyBoardsError> {
        let data: &EditEmoji = &self;

        // only admins should be editing emojis
        let _view = require_user(context.pool(), context.master_key(), auth)
            .await
            .require_admin(AdminPerms::Config)
            .unwrap()?;

        let local_site_id = Some(LocalSite::read(context.pool()).await?.id);
        let shortcode = Some(data.shortcode.clone());
        let alt_text = Some(data.alt_text.clone());
        let image_url = Some(data.image_url.clone());
        let category = Some(data.category.clone());
        let updated = Some(naive_now());

        let emoji_form = EmojiForm {
            local_site_id,
            shortcode,
            alt_text,
            image_url,
            category,
            updated,
        };

        let updated_emoji = Emoji::update(context.pool(), data.id.clone(), &emoji_form).await?;
        EmojiKeyword::delete(context.pool(), data.id.clone()).await?;

        let mut keywords = vec![];

        for keyword in data.keywords.clone() {
            let keyword_form = EmojiKeywordForm {
                emoji_id: Some(updated_emoji.id.clone()),
                keyword: Some(keyword.to_lowercase().trim().to_string().clone()),
            };
            keywords.push(keyword_form);
        }
        EmojiKeyword::create(context.pool(), keywords).await?;

        let emoji_view = EmojiView::get(context.pool(), updated_emoji.id.clone()).await?;

        Ok(EmojiResponse { emoji: emoji_view })
    }
}
