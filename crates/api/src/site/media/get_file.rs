use crate::Perform;
use std::fs;
use actix_web::web::Data;
use tinyboards_api_common::{
    data::TinyBoardsContext,
    site::{GetFile, FileNamePath},
};
use tinyboards_db::{models::site::{uploads::Upload}, traits::Crud};
use tinyboards_utils::error::TinyBoardsError;

#[async_trait::async_trait(?Send)]
impl<'des> Perform<'des> for GetFile {
    type Response = ();
    type Route = FileNamePath;

    #[tracing::instrument(skip(context))]
    async fn perform(
        self,
        context: &Data<TinyBoardsContext>,
        path: Self::Route,
        _: Option<&str>,
    ) -> Result<(), TinyBoardsError> {
        let file_to_get = path.file_name.clone();

        let file = Upload::find_by_name(context.pool(), &file_to_delete).await?;

        let file_path = file.file_path.clone();

        todo!();
    }
}
