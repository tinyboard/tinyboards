// use async_graphql::*;
// use tinyboards_db::models::person::{
//     local_user::LocalUser as DbLocalUser,
//     local_user::*,
//     person::*
// };
// use tinyboards_db::models::post::*;
// use tinyboards_db::newtypes::DbUrl;
// use chrono::NaiveDateTime;
// use tinyboards_db::traits::Crud;
// use tinyboards_utils::TinyBoardsError;
// use url::Url;

// #[derive(Default)]
// pub struct Post {
//     pub id: i32,
//     pub title: String,
//     pub type_: String,
//     pub url: Option<DbUrl>,
//     pub thumbnail_url: Option<DbUrl>,
//     pub permalink: Option<DbUrl>,
//     pub body: String,
//     pub body_html: String,
//     pub creator_id: i32,
//     pub board_id: i32,
//     pub is_removed: bool,
//     pub is_locked: bool,
//     pub creation_date: NaiveDateTime,
//     pub is_deleted: bool,
//     pub is_nsfw: bool,
//     pub updated: Option<NaiveDateTime>,
//     pub image: Option<DbUrl>,
//     pub language_id: i32,
//     pub ap_id: Option<DbUrl>,
//     pub local: bool,
//     pub featured_board: bool,
//     pub featured_local: bool,
//     pub title_chunk: String,
// }


// #[Object]
// impl Post {
//     pub async fn submit(
//         &self,
//         ctx: &Context<'_>,
//         form: PostForm,
//     ) -> Result<Self, TinyBoardsError> {
//         let v = ctx.data_unchecked::<LoggedInUser>
//     }
// }