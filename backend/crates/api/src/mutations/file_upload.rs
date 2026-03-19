use crate::{LoggedInUser, Settings, helpers::files::upload::upload_file_opendal};
use async_graphql::*;

#[derive(Default)]
pub struct FileUploadMutation;

#[Object]
impl FileUploadMutation {
    /// Upload a file and return the URL
    ///
    /// This creates an Upload record in the database but does NOT create a ContentUpload link.
    /// When the user actually submits their comment/post, the content should be scanned for
    /// image URLs and ContentUpload records should be created to link them.
    ///
    /// Orphaned uploads (uploads not linked to any content after a certain time) are cleaned
    /// up by the orphaned upload cleanup system.
    async fn upload_file(
        &self,
        ctx: &Context<'_>,
        file: Upload,
    ) -> Result<String> {
        let user = ctx.data::<LoggedInUser>()?.require_user_not_banned()?;
        let settings = ctx.data::<Settings>()?.as_ref();

        // Upload file and create Upload record
        let url = upload_file_opendal(
            file,
            None,
            user.id,
            Some(settings.media.max_file_size_mb),
            ctx,
        )
        .await?;

        Ok(url.to_string())
    }
}
